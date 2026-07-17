mod default_request_password_reset;
mod request_password_reset_input;

use crate::business::{
    entities::user::user_gateway::UserGateway,
    error::{GatewayError, ResetPasswordRequestTooEarlyError, UserNotFoundError, use_case_error},
    services::ResetPasswordCodeGenerator,
};
use jsonrpc_usecase::UseCase;
use std::sync::Arc;

pub use request_password_reset_input::RequestPasswordResetInput;

pub struct RequestPasswordReset {
    user_gateway: Arc<dyn UserGateway>,
    reset_password_code_generator: Arc<dyn ResetPasswordCodeGenerator>,
}

use_case_error! {
    pub enum RequestPasswordResetError {
        UserNotFound(UserNotFoundError),
        ResetPasswordRequestTooEarly(ResetPasswordRequestTooEarlyError),
        Gateway(GatewayError),
    }
}

#[UseCase]
impl RequestPasswordReset {
    async fn execute(
        &self,
        input: RequestPasswordResetInput,
    ) -> Result<(), RequestPasswordResetError> {
        let mut user = self
            .user_gateway
            .find_one_by_email_or_username(&input.subject, &input.subject)
            .await?
            .ok_or(UserNotFoundError)?;

        if !user.is_reset_password_request_cooldown_expired() {
            let remaining_minutes = user
                .reset_password_request_remaining_cooldown_minutes()
                .unwrap_or(1);

            return Err(ResetPasswordRequestTooEarlyError::new(remaining_minutes).into());
        }

        let reset_password_code = self.reset_password_code_generator.generate();
        user.define_reset_password_code(reset_password_code);
        self.user_gateway.save(&user).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::request_password_reset_input::RequestPasswordResetInput;
    use super::{RequestPasswordReset, RequestPasswordResetError};
    use crate::business::{
        EntityBuilder,
        entities::user::{
            User, UserBuilder,
            user_gateway::{InMemoryUserGateway, UserGateway},
        },
        services::reset_password_code_generator::ResetPasswordCodeGeneratorDouble,
    };
    use serde_json::json;
    use std::sync::Arc;

    #[tokio::test]
    async fn returns_user_not_found_when_user_does_not_exist() {
        let user_gateway = Arc::new(InMemoryUserGateway::default());
        let reset_password_code_generator = Arc::new(ResetPasswordCodeGeneratorDouble::generating(
            "reset-password-code",
        ));
        let use_case = RequestPasswordReset {
            user_gateway,
            reset_password_code_generator: reset_password_code_generator.clone(),
        };

        let error = use_case
            .execute(request_password_reset_input("alice"))
            .await
            .expect_err("unknown user should be rejected");

        assert!(matches!(error, RequestPasswordResetError::UserNotFound(_)));
        assert_eq!(reset_password_code_generator.calls(), 0);
    }

    #[tokio::test]
    async fn returns_too_early_when_latest_request_is_less_than_two_hours_old() {
        let user_gateway = Arc::new(InMemoryUserGateway::default());
        let mut user = UserBuilder::init()
            .with_username("alice".to_owned())
            .with_email("alice@example.com".to_owned())
            .with_password("hashed-password".to_owned())
            .build();
        user.define_reset_password_code("existing-reset-password-code".to_owned());
        saved_user(user_gateway.as_ref(), user.clone()).await;
        let reset_password_code_generator = Arc::new(ResetPasswordCodeGeneratorDouble::generating(
            "reset-password-code",
        ));
        let use_case = RequestPasswordReset {
            user_gateway: user_gateway.clone(),
            reset_password_code_generator: reset_password_code_generator.clone(),
        };

        let error = use_case
            .execute(request_password_reset_input("alice"))
            .await
            .expect_err("recent reset password request should be rejected");

        assert!(matches!(
            error,
            RequestPasswordResetError::ResetPasswordRequestTooEarly(_)
        ));
        if let RequestPasswordResetError::ResetPasswordRequestTooEarly(error) = error {
            assert_eq!(error.remaining_minutes(), 120);
        }
        assert_eq!(reset_password_code_generator.calls(), 0);
        assert_eq!(
            user_gateway
                .find_one_by_email_or_username("alice@example.com", "alice")
                .await
                .expect("user lookup should succeed"),
            Some(user)
        );
    }

    #[tokio::test]
    async fn generates_reset_password_code_saves_user_and_returns_unit() {
        let user_gateway = Arc::new(InMemoryUserGateway::default());
        saved_user(
            user_gateway.as_ref(),
            UserBuilder::init()
                .with_username("alice".to_owned())
                .with_email("alice@example.com".to_owned())
                .with_password("hashed-password".to_owned())
                .build(),
        )
        .await;
        let reset_password_code_generator = Arc::new(ResetPasswordCodeGeneratorDouble::generating(
            "reset-password-code",
        ));
        let use_case = RequestPasswordReset {
            user_gateway: user_gateway.clone(),
            reset_password_code_generator: reset_password_code_generator.clone(),
        };

        use_case
            .execute(request_password_reset_input("ALICE@EXAMPLE.COM"))
            .await
            .expect("reset password request should succeed");

        let saved_user = user_gateway
            .find_one_by_email_or_username("alice@example.com", "alice")
            .await
            .expect("saved user lookup should succeed")
            .expect("user should still exist");

        assert_eq!(reset_password_code_generator.calls(), 1);
        assert_eq!(
            saved_user.reset_password_code(),
            Some("reset-password-code")
        );
        assert!(!saved_user.is_reset_password_request_cooldown_expired());
    }

    async fn saved_user(user_gateway: &dyn UserGateway, user: User) -> User {
        user_gateway
            .save(&user)
            .await
            .expect("user should be saved");

        user
    }

    fn request_password_reset_input(subject: &str) -> RequestPasswordResetInput {
        serde_json::from_value(json!({
            "subject": subject,
        }))
        .expect("request password reset input should deserialize")
    }
}
