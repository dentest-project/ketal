mod default_send_reset_password_request_email;

use crate::{
    business::{
        entities::user::user_gateway::UserGateway,
        error::{GatewayError, UserNotFoundError},
        usecases::request_password_reset::RequestPasswordResetInput,
    },
    infrastructure::communication::email::{ResetPasswordRequestMail, Sender, SenderError},
};
use jsonrpc_usecase::{UseCaseEvent, UseCaseEventConsumer};
use std::sync::Arc;
use thiserror::Error;

pub struct SendResetPasswordRequestEmail {
    user_gateway: Arc<dyn UserGateway>,
    sender: Arc<dyn Sender>,
    allowed_origin: String,
}

impl SendResetPasswordRequestEmail {
    pub fn new(
        user_gateway: Arc<dyn UserGateway>,
        sender: Arc<dyn Sender>,
        allowed_origin: String,
    ) -> Self {
        Self {
            user_gateway,
            sender,
            allowed_origin,
        }
    }

    async fn send_reset_password_request_email(
        &self,
        subject: &str,
    ) -> Result<(), SendResetPasswordRequestEmailError> {
        let user = self
            .user_gateway
            .find_one_by_email_or_username(subject, subject)
            .await?
            .ok_or(UserNotFoundError)?;
        let reset_password_code = user
            .reset_password_code()
            .ok_or(SendResetPasswordRequestEmailError::MissingResetPasswordCode)?;
        let mail = ResetPasswordRequestMail::new(reset_password_link(
            &self.allowed_origin,
            reset_password_code,
        ));

        self.sender.send(&mail, user.email().to_owned()).await?;

        Ok(())
    }
}

#[derive(Debug, Error)]
enum SendResetPasswordRequestEmailError {
    #[error(transparent)]
    UserNotFound(#[from] UserNotFoundError),
    #[error("reset password code is missing")]
    MissingResetPasswordCode,
    #[error(transparent)]
    Gateway(#[from] GatewayError),
    #[error(transparent)]
    Sender(#[from] SenderError),
}

#[UseCaseEventConsumer(event = "DidRequestPasswordReset")]
impl SendResetPasswordRequestEmail {
    async fn consume(&self, event: &UseCaseEvent) {
        let Some(input) = event.get_input::<RequestPasswordResetInput>() else {
            eprintln!("DidRequestPasswordReset event input is missing or has an unexpected type");
            return;
        };

        if let Err(error) = self.send_reset_password_request_email(&input.subject).await {
            eprintln!("failed to send reset password request email: {error}");
        }
    }
}

fn reset_password_link(allowed_origin: &str, reset_password_code: &str) -> String {
    format!(
        "{}/reset-password?code={reset_password_code}",
        allowed_origin.trim_end_matches('/')
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        business::{
            EntityBuilder,
            entities::user::{
                UserBuilder,
                user_gateway::{InMemoryUserGateway, UserGateway},
            },
        },
        infrastructure::communication::email::{Mail, SenderFuture},
    };
    use std::sync::Mutex;

    #[derive(Default)]
    struct FakeSender {
        sent: Mutex<Vec<SentMail>>,
    }

    #[derive(Debug, PartialEq, Eq)]
    struct SentMail {
        to: String,
        subject: String,
        plain: String,
    }

    impl FakeSender {
        fn sent(&self) -> Vec<SentMail> {
            self.sent
                .lock()
                .expect("sent mail mutex should not be poisoned")
                .iter()
                .map(|mail| SentMail {
                    to: mail.to.clone(),
                    subject: mail.subject.clone(),
                    plain: mail.plain.clone(),
                })
                .collect()
        }
    }

    impl Sender for FakeSender {
        fn send<'a>(&'a self, mail: &'a dyn Mail, to: String) -> SenderFuture<'a> {
            Box::pin(async move {
                self.sent
                    .lock()
                    .expect("sent mail mutex should not be poisoned")
                    .push(SentMail {
                        to,
                        subject: mail.subject(),
                        plain: mail.plain(),
                    });

                Ok(())
            })
        }
    }

    #[tokio::test]
    async fn sends_reset_password_request_email_to_user_found_by_subject() {
        let user_gateway = Arc::new(InMemoryUserGateway::default());
        let sender = Arc::new(FakeSender::default());
        let mut user = UserBuilder::init()
            .with_username("alice".to_owned())
            .with_email("alice@example.com".to_owned())
            .with_password("hashed-password".to_owned())
            .build();
        user.define_reset_password_code("reset-password-code".to_owned());
        user_gateway
            .save(&user)
            .await
            .expect("user should be saved");
        let reaction = SendResetPasswordRequestEmail::new(
            user_gateway,
            sender.clone(),
            "http://localhost:5173/".to_owned(),
        );

        reaction
            .send_reset_password_request_email("ALICE@EXAMPLE.COM")
            .await
            .expect("reaction should send reset password request email");

        assert_eq!(
            sender.sent(),
            vec![SentMail {
                to: "alice@example.com".to_owned(),
                subject: "Dentest: reset your password".to_owned(),
                plain: "Hello,\n\n\
                        Click on the following link in order to reset your password: http://localhost:5173/reset-password?code=reset-password-code\n\n\
                        Have a nice day!"
                    .to_owned(),
            }]
        );
    }

    #[test]
    fn builds_reset_password_link_from_allowed_origin_and_code() {
        assert_eq!(
            reset_password_link("http://localhost:5173/", "reset-password-code"),
            "http://localhost:5173/reset-password?code=reset-password-code"
        );
    }
}
