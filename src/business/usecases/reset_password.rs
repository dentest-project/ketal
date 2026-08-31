mod default_reset_password;
mod reset_password_input;

use crate::business::{
    entities::user::user_gateway::UserGateway,
    error::{GatewayError, PasswordEncoderError, UserNotFoundError, use_case_error},
    services::PasswordEncoder,
    usecases::outputs::user::UserDetailedOutput,
};
use jsonrpc_usecase::UseCase;
use std::sync::Arc;

pub use reset_password_input::ResetPasswordInput;

pub struct ResetPassword {
    user_gateway: Arc<dyn UserGateway>,
    password_encoder: Arc<dyn PasswordEncoder>,
}

use_case_error! {
    pub enum ResetPasswordError {
        UserNotFound(UserNotFoundError),
        Gateway(GatewayError),
        PasswordEncoder(PasswordEncoderError),
    }
}

#[UseCase]
impl ResetPassword {
    async fn execute(
        &self,
        input: ResetPasswordInput,
    ) -> Result<UserDetailedOutput, ResetPasswordError> {
        let mut user = self
            .user_gateway
            .find_one_by_reset_password_code(&input.code)
            .await?
            .ok_or(UserNotFoundError)?;
        let encoded_password = self.password_encoder.encode(&input.password)?;

        user.reset_password(encoded_password);
        self.user_gateway.save(&user).await?;

        Ok((&user).into())
    }
}

#[cfg(test)]
mod tests {
    use super::{ResetPassword, ResetPasswordError, ResetPasswordInput};
    use crate::business::{
        EntityBuilder,
        entities::user::{
            UserBuilder,
            user_gateway::{InMemoryUserGateway, UserGateway},
        },
        services::password_encoder::PasswordEncoderDouble,
    };
    use jsonrpc_usecase::UseCaseExecutionError;
    use serde_json::json;
    use std::sync::Arc;

    #[tokio::test]
    async fn returns_user_not_found_when_reset_password_code_does_not_exist() {
        let user_gateway = Arc::new(InMemoryUserGateway::default());
        let password_encoder = Arc::new(PasswordEncoderDouble::encoding("encoded-password"));
        let use_case = ResetPassword {
            user_gateway,
            password_encoder: password_encoder.clone(),
        };

        let error = use_case
            .execute(reset_password_input("unknown-code", "new-password"))
            .await
            .expect_err("unknown reset password code should be rejected");

        assert!(matches!(
            error,
            UseCaseExecutionError::Execution(ResetPasswordError::UserNotFound(_))
        ));
        assert!(password_encoder.received_password().is_none());
    }

    #[tokio::test]
    async fn encodes_password_and_saves_user_found_by_reset_password_code() {
        let user_gateway = Arc::new(InMemoryUserGateway::default());
        let mut user = UserBuilder::init()
            .with_username("alice".to_owned())
            .with_email("alice@example.com".to_owned())
            .with_password("old-encoded-password".to_owned())
            .build();
        user.define_reset_password_code("reset-password-code".to_owned());
        user_gateway
            .save(&user)
            .await
            .expect("user should be saved");
        let password_encoder = Arc::new(PasswordEncoderDouble::encoding("new-encoded-password"));
        let use_case = ResetPassword {
            user_gateway: user_gateway.clone(),
            password_encoder: password_encoder.clone(),
        };

        let output = use_case
            .execute(reset_password_input("reset-password-code", "new-password"))
            .await
            .expect("password reset should succeed");

        let saved_user = user_gateway
            .find_one_by_email_or_username("alice@example.com", "alice")
            .await
            .expect("saved user lookup should succeed")
            .expect("user should still exist");

        assert_eq!(
            password_encoder.received_password().as_deref(),
            Some("new-password")
        );
        assert_eq!(saved_user.password(), "new-encoded-password");
        assert_eq!(saved_user.reset_password_code(), None);
        assert_eq!(output.id, saved_user.id().to_string());
        assert_eq!(output.username, "alice");
        assert_eq!(output.email, "alice@example.com");
    }

    fn reset_password_input(code: &str, password: &str) -> ResetPasswordInput {
        serde_json::from_value(json!({
            "code": code,
            "password": password,
        }))
        .expect("reset password input should deserialize")
    }
}
