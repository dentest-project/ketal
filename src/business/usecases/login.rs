mod default_login;
mod login_input;
mod login_output;

use crate::business::{
    entities::user::user_gateway::UserGateway,
    error::{
        GatewayError, InvalidCredentialsError, PasswordDecoderError, TokenGeneratorError,
        use_case_error,
    },
    services::{PasswordDecoder, TokenGenerator},
};
use jsonrpc_usecase::UseCase;
use std::sync::Arc;

pub use login_input::LoginInput;
pub use login_output::LoginOutput;

pub struct Login {
    user_gateway: Arc<dyn UserGateway>,
    password_decoder: Arc<dyn PasswordDecoder>,
    token_generator: Arc<dyn TokenGenerator>,
}

use_case_error! {
    pub enum LoginError {
        InvalidCredentials(InvalidCredentialsError),
        Gateway(GatewayError),
        PasswordDecoder(PasswordDecoderError),
        TokenGenerator(TokenGeneratorError),
    }
}

#[UseCase]
impl Login {
    async fn execute(&self, input: LoginInput) -> Result<LoginOutput, LoginError> {
        let user = self
            .user_gateway
            .find_one_by_email_or_username(&input.subject, &input.subject)
            .await?
            .ok_or(InvalidCredentialsError)?;

        if !self
            .password_decoder
            .matches(&input.password, user.password())?
        {
            return Err(InvalidCredentialsError.into());
        }

        Ok(LoginOutput {
            token: self.token_generator.generate_for(&user)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::login_input::LoginInput;
    use super::{Login, LoginError};
    use crate::business::{
        EntityBuilder,
        entities::user::{
            User, UserBuilder,
            user_gateway::{InMemoryUserGateway, UserGateway},
        },
        services::{
            password_decoder::{
                Argon2PasswordDecoder, BcryptPasswordDecoder, MultiplePasswordDecoder,
                PasswordDecoderDouble,
            },
            token_generator::TokenGeneratorDouble,
        },
    };
    use serde_json::json;
    use std::sync::Arc;

    const TEST_BCRYPT_COST: u32 = 4;

    #[tokio::test]
    async fn returns_invalid_credentials_when_user_does_not_exist() {
        let user_gateway = Arc::new(InMemoryUserGateway::default());
        let password_decoder = Arc::new(PasswordDecoderDouble::matching(true));
        let token_generator = Arc::new(TokenGeneratorDouble::new("token"));
        let login = Login {
            user_gateway,
            password_decoder: password_decoder.clone(),
            token_generator: token_generator.clone(),
        };
        let input = login_input("ALICE", "secret123");

        let error = login
            .execute(input)
            .await
            .expect_err("unknown user should be rejected");

        assert!(matches!(error, LoginError::InvalidCredentials(_)));
        assert!(password_decoder.received_matches().is_empty());
        assert!(token_generator.received_usernames().is_empty());
    }

    #[tokio::test]
    async fn returns_invalid_credentials_when_password_does_not_match() {
        let user_gateway = Arc::new(InMemoryUserGateway::default());
        let user = saved_user(user_gateway.as_ref()).await;
        let password_decoder = Arc::new(PasswordDecoderDouble::matching(false));
        let token_generator = Arc::new(TokenGeneratorDouble::new("token"));
        let login = Login {
            user_gateway,
            password_decoder: password_decoder.clone(),
            token_generator: token_generator.clone(),
        };
        let input = login_input("alice", "secret123");

        let error = login
            .execute(input)
            .await
            .expect_err("wrong password should be rejected");

        assert!(matches!(error, LoginError::InvalidCredentials(_)));
        assert_eq!(
            password_decoder.received_matches(),
            vec![("secret123".to_owned(), user.password().to_owned())]
        );
        assert!(token_generator.received_usernames().is_empty());
    }

    #[tokio::test]
    async fn returns_token_when_credentials_match_with_username() {
        let user_gateway = Arc::new(InMemoryUserGateway::default());
        saved_user(user_gateway.as_ref()).await;
        let password_decoder = Arc::new(PasswordDecoderDouble::matching(true));
        let token_generator = Arc::new(TokenGeneratorDouble::new("generated-token"));
        let login = Login {
            user_gateway,
            password_decoder: password_decoder.clone(),
            token_generator: token_generator.clone(),
        };
        let input = login_input("alice", "secret123");

        let output = login.execute(input).await.expect("login should succeed");

        assert_eq!(output.token, "generated-token");
        assert_eq!(
            token_generator.received_usernames(),
            vec!["alice".to_owned()]
        );
    }

    #[tokio::test]
    async fn returns_token_when_credentials_match_with_email() {
        let user_gateway = Arc::new(InMemoryUserGateway::default());
        saved_user(user_gateway.as_ref()).await;
        let password_decoder = Arc::new(PasswordDecoderDouble::matching(true));
        let token_generator = Arc::new(TokenGeneratorDouble::new("generated-token"));
        let login = Login {
            user_gateway,
            password_decoder,
            token_generator,
        };
        let input = login_input("ALICE@EXAMPLE.COM", "secret123");

        let output = login.execute(input).await.expect("login should succeed");

        assert_eq!(output.token, "generated-token");
    }

    #[tokio::test]
    async fn returns_token_when_bcrypt_password_matches() {
        let user_gateway = Arc::new(InMemoryUserGateway::default());
        saved_user_with_password(
            user_gateway.as_ref(),
            bcrypt::hash("secret123", TEST_BCRYPT_COST).expect("password should hash"),
        )
        .await;
        let password_decoder = Arc::new(MultiplePasswordDecoder::new(vec![
            Arc::new(Argon2PasswordDecoder::new()),
            Arc::new(BcryptPasswordDecoder::new()),
        ]));
        let token_generator = Arc::new(TokenGeneratorDouble::new("generated-token"));
        let login = Login {
            user_gateway,
            password_decoder,
            token_generator,
        };
        let input = login_input("alice", "secret123");

        let output = login
            .execute(input)
            .await
            .expect("bcrypt login should succeed");

        assert_eq!(output.token, "generated-token");
    }

    async fn saved_user(user_gateway: &dyn UserGateway) -> User {
        saved_user_with_password(user_gateway, "hashed-password".to_owned()).await
    }

    async fn saved_user_with_password(user_gateway: &dyn UserGateway, password: String) -> User {
        let user = UserBuilder::init()
            .with_username("alice".to_owned())
            .with_email("alice@example.com".to_owned())
            .with_password(password)
            .build();

        user_gateway
            .save(&user)
            .await
            .expect("user should be saved");

        user
    }

    fn login_input(subject: &str, password: &str) -> LoginInput {
        serde_json::from_value(json!({
            "subject": subject,
            "password": password
        }))
        .expect("login input should deserialize")
    }
}
