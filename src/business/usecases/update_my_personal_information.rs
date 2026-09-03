mod default_update_my_personal_information;
mod update_my_personal_information_input;

use crate::{
    business::{
        RequestContext,
        entities::user::{User, user_gateway::UserGateway},
        error::{
            GatewayError, PasswordEncoderError, UnexpectedError, UserAlreadyExistsError,
            use_case_error,
        },
        services::PasswordEncoder,
        usecases::outputs::user::UserDetailedOutput,
    },
    infrastructure::guards::AuthenticatedUserGuard,
};
use jsonrpc_usecase::{UseCase, current_context};
use std::sync::Arc;

pub use update_my_personal_information_input::UpdateMyPersonalInformationInput;

pub struct UpdateMyPersonalInformation {
    user_gateway: Arc<dyn UserGateway>,
    password_encoder: Arc<dyn PasswordEncoder>,
}

use_case_error! {
    pub enum UpdateMyPersonalInformationError {
        UserAlreadyExists(UserAlreadyExistsError),
        Gateway(GatewayError),
        PasswordEncoder(PasswordEncoderError),
        Unexpected(UnexpectedError),
    }
}

#[UseCase(guards = [AuthenticatedUserGuard])]
impl UpdateMyPersonalInformation {
    async fn execute(
        &self,
        input: UpdateMyPersonalInformationInput,
    ) -> Result<UserDetailedOutput, UpdateMyPersonalInformationError> {
        let context = current_context::<RequestContext>().ok_or(UnexpectedError)?;
        let user = context.user.clone().ok_or(UnexpectedError)?;

        self.update_user(user, input).await
    }

    async fn update_user(
        &self,
        mut user: User,
        input: UpdateMyPersonalInformationInput,
    ) -> Result<UserDetailedOutput, UpdateMyPersonalInformationError> {
        if self
            .user_gateway
            .find_one_by_email_or_username_excluding_user(&input.email, &input.username, &user)
            .await?
            .is_some()
        {
            return Err(UserAlreadyExistsError.into());
        }

        let encoded_password = input
            .password
            .as_deref()
            .map(|password| self.password_encoder.encode(password))
            .transpose()?;

        user.update_personal_information(input.username, input.email, encoded_password);
        self.user_gateway.save(&user).await?;

        Ok((&user).into())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        UpdateMyPersonalInformation, UpdateMyPersonalInformationError,
        UpdateMyPersonalInformationInput,
    };
    use crate::business::{
        EntityBuilder,
        entities::user::{
            User, UserBuilder,
            user_gateway::{InMemoryUserGateway, UserGateway},
        },
        services::password_encoder::PasswordEncoderDouble,
    };
    use jsonrpc_usecase::UseCaseExecutionError;
    use std::{error::Error, io, sync::Arc};

    #[tokio::test]
    async fn returns_unexpected_error_when_request_context_is_missing() {
        let use_case = update_use_case(
            Arc::new(InMemoryUserGateway::default()),
            Arc::new(PasswordEncoderDouble::encoding("unused-encoded-password")),
        );

        let result = use_case
            .execute(update_input("alice", "alice@example.com", None))
            .await;

        assert!(matches!(
            result,
            Err(UseCaseExecutionError::Execution(
                UpdateMyPersonalInformationError::Unexpected(_)
            ))
        ));
    }

    #[tokio::test]
    async fn updates_username_and_email_without_changing_an_omitted_password()
    -> Result<(), Box<dyn Error>> {
        let user_gateway = Arc::new(InMemoryUserGateway::default());
        let user = user("alice", "alice@example.com", "old-encoded-password");
        user_gateway.save(&user).await?;
        let password_encoder = Arc::new(PasswordEncoderDouble::encoding("unused-password"));
        let use_case = update_use_case(user_gateway.clone(), password_encoder.clone());

        let output = use_case
            .update_user(
                user,
                update_input("alice-updated", "alice-updated@example.com", None),
            )
            .await?;

        assert_eq!(output.username, "alice-updated");
        assert_eq!(output.email, "alice-updated@example.com");
        assert!(password_encoder.received_password().is_none());

        let saved_user = user_gateway
            .find_one_by_email_or_username("alice-updated@example.com", "alice-updated")
            .await?
            .ok_or_else(|| io::Error::other("updated user should be saved"))?;
        assert_eq!(saved_user.username(), "alice-updated");
        assert_eq!(saved_user.email(), "alice-updated@example.com");
        assert_eq!(saved_user.password(), "old-encoded-password");

        Ok(())
    }

    #[tokio::test]
    async fn encodes_and_changes_a_provided_password() -> Result<(), Box<dyn Error>> {
        let user_gateway = Arc::new(InMemoryUserGateway::default());
        let user = user("alice", "alice@example.com", "old-encoded-password");
        user_gateway.save(&user).await?;
        let password_encoder = Arc::new(PasswordEncoderDouble::encoding("new-encoded-password"));
        let use_case = update_use_case(user_gateway.clone(), password_encoder.clone());

        use_case
            .update_user(
                user,
                update_input("alice", "alice@example.com", Some("new-password")),
            )
            .await?;

        assert_eq!(
            password_encoder.received_password().as_deref(),
            Some("new-password")
        );

        let saved_user = user_gateway
            .find_one_by_email_or_username("alice@example.com", "alice")
            .await?
            .ok_or_else(|| io::Error::other("updated user should be saved"))?;
        assert_eq!(saved_user.password(), "new-encoded-password");

        Ok(())
    }

    #[tokio::test]
    async fn rejects_information_owned_by_another_user() -> Result<(), Box<dyn Error>> {
        let user_gateway = Arc::new(InMemoryUserGateway::default());
        let authenticated_user = user("alice", "alice@example.com", "alice-password");
        let other_user = user("bob", "bob@example.com", "bob-password");
        user_gateway.save(&authenticated_user).await?;
        user_gateway.save(&other_user).await?;
        let password_encoder = Arc::new(PasswordEncoderDouble::encoding("unused-password"));
        let use_case = update_use_case(user_gateway, password_encoder.clone());

        let result = use_case
            .update_user(
                authenticated_user,
                update_input("alice", "BOB@EXAMPLE.COM", Some("new-password")),
            )
            .await;

        assert!(matches!(
            result,
            Err(UpdateMyPersonalInformationError::UserAlreadyExists(_))
        ));
        assert!(password_encoder.received_password().is_none());

        Ok(())
    }

    fn update_use_case(
        user_gateway: Arc<dyn UserGateway>,
        password_encoder: Arc<PasswordEncoderDouble>,
    ) -> UpdateMyPersonalInformation {
        UpdateMyPersonalInformation {
            user_gateway,
            password_encoder,
        }
    }

    fn user(username: &str, email: &str, password: &str) -> User {
        UserBuilder::init()
            .with_username(username.to_owned())
            .with_email(email.to_owned())
            .with_password(password.to_owned())
            .build()
    }

    fn update_input(
        username: &str,
        email: &str,
        password: Option<&str>,
    ) -> UpdateMyPersonalInformationInput {
        UpdateMyPersonalInformationInput {
            username: username.to_owned(),
            email: email.to_owned(),
            password: password.map(str::to_owned),
        }
    }
}
