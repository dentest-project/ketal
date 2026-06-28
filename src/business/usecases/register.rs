mod default_register;
mod register_input;

use crate::business::{
    EntityBuilder,
    entities::user::{UserBuilder, user_gateway::UserGateway},
    error::{GatewayError, PasswordEncoderError, UserAlreadyExistsError, use_case_error},
    services::PasswordEncoder,
    usecases::outputs::user::UserDetailedOutput,
};
use jsonrpc_usecase::UseCase;
use std::sync::Arc;

pub use register_input::RegisterInput;

pub struct Register {
    user_gateway: Arc<dyn UserGateway>,
    password_encoder: Arc<dyn PasswordEncoder>,
}

use_case_error! {
    pub enum RegisterError {
        UserAlreadyExists(UserAlreadyExistsError),
        Gateway(GatewayError),
        PasswordEncoder(PasswordEncoderError),
    }
}

#[UseCase]
impl Register {
    async fn execute(&self, input: RegisterInput) -> Result<UserDetailedOutput, RegisterError> {
        if self
            .user_gateway
            .find_one_by_email_or_username(&input.email, &input.username)
            .await?
            .is_some()
        {
            return Err(UserAlreadyExistsError.into());
        }

        let hashed_password = self.password_encoder.encode(&input.password)?;
        let RegisterInput {
            username,
            email,
            password: _,
        } = input;

        let user = UserBuilder::init()
            .with_username(username)
            .with_email(email)
            .with_password(hashed_password)
            .build();

        self.user_gateway.save(&user).await?;

        Ok((&user).into())
    }
}

#[cfg(test)]
mod tests {
    use super::register_input::RegisterInput;
    use super::{Register, RegisterError};
    use crate::business::{
        EntityBuilder,
        entities::user::{
            UserBuilder,
            user_gateway::{InMemoryUserGateway, UserGateway},
        },
        services::{PasswordEncoder, PasswordEncoderResult},
    };
    use serde_json::json;
    use std::sync::{Arc, Mutex};

    struct FakePasswordEncoder {
        encoded_password: String,
        received_password: Mutex<Option<String>>,
    }

    impl FakePasswordEncoder {
        fn new(encoded_password: impl Into<String>) -> Self {
            Self {
                encoded_password: encoded_password.into(),
                received_password: Mutex::new(None),
            }
        }

        fn received_password(&self) -> Option<String> {
            self.received_password
                .lock()
                .expect("received password mutex should not be poisoned")
                .clone()
        }
    }

    impl PasswordEncoder for FakePasswordEncoder {
        fn encode(&self, password: &str) -> PasswordEncoderResult<String> {
            self.received_password
                .lock()
                .expect("received password mutex should not be poisoned")
                .replace(password.to_owned());

            Ok(self.encoded_password.clone())
        }
    }

    #[tokio::test]
    async fn returns_user_already_exists_when_email_or_username_is_taken() {
        let user_gateway = Arc::new(InMemoryUserGateway::default());
        let existing_user = UserBuilder::init()
            .with_username("alice".to_owned())
            .with_email("alice@example.com".to_owned())
            .with_password("hashed-password".to_owned())
            .build();
        let password_encoder = Arc::new(FakePasswordEncoder::new("hashed-password"));
        let register = Register {
            user_gateway: user_gateway.clone(),
            password_encoder: password_encoder.clone(),
        };
        let input: RegisterInput = serde_json::from_value(json!({
            "username": "alice",
            "email": "alice@example.com",
            "password": "secret123"
        }))
        .expect("register input should deserialize");

        user_gateway
            .save(&existing_user)
            .await
            .expect("existing user should be stored");

        let error = register
            .execute(input)
            .await
            .expect_err("duplicate user should be rejected");

        assert!(matches!(error, RegisterError::UserAlreadyExists(_)));
        assert!(password_encoder.received_password().is_none());
    }

    #[tokio::test]
    async fn hashes_password_saves_user_and_returns_detailed_output() {
        let user_gateway = Arc::new(InMemoryUserGateway::default());
        let password_encoder = Arc::new(FakePasswordEncoder::new("hashed-password"));
        let register = Register {
            user_gateway: user_gateway.clone(),
            password_encoder: password_encoder.clone(),
        };
        let input: RegisterInput = serde_json::from_value(json!({
            "username": "  alice  ",
            "email": "  alice@example.com  ",
            "password": "  secret123  "
        }))
        .expect("register input should deserialize");

        let output = register
            .execute(input)
            .await
            .expect("register should succeed");
        let saved_user = user_gateway
            .find_one_by_email_or_username("alice@example.com", "alice")
            .await
            .expect("saved user lookup should succeed")
            .expect("registered user should be saved");

        assert_eq!(
            password_encoder.received_password().as_deref(),
            Some("secret123")
        );
        assert_eq!(saved_user.username(), "alice");
        assert_eq!(saved_user.email(), "alice@example.com");
        assert_eq!(saved_user.password(), "hashed-password");
        assert_eq!(output.id, saved_user.id().to_string());
        assert_eq!(output.username, "alice");
        assert_eq!(output.email, "alice@example.com");
    }
}
