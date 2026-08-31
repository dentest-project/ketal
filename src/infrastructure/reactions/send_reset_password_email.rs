mod default_send_reset_password_email;

use crate::{
    business::usecases::outputs::user::UserDetailedOutput,
    infrastructure::communication::email::{ResetPasswordMail, Sender, SenderError},
};
use jsonrpc_usecase::{UseCaseEvent, UseCaseEventConsumer};
use std::sync::Arc;

pub struct SendResetPasswordEmail {
    sender: Arc<dyn Sender>,
}

impl SendResetPasswordEmail {
    pub fn new(sender: Arc<dyn Sender>) -> Self {
        Self { sender }
    }

    async fn send_reset_password_email(
        &self,
        user: &UserDetailedOutput,
    ) -> Result<(), SenderError> {
        let mail = ResetPasswordMail::new();

        self.sender.send(&mail, user.email.clone()).await
    }
}

#[UseCaseEventConsumer(event = "DidResetPassword")]
impl SendResetPasswordEmail {
    async fn consume(&self, event: &UseCaseEvent) {
        let Some(user) = event.get_output::<UserDetailedOutput>() else {
            eprintln!("DidResetPassword event output is missing or has an unexpected type");
            return;
        };

        if let Err(error) = self.send_reset_password_email(user).await {
            eprintln!("failed to send reset password email: {error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::communication::email::{Mail, SenderFuture};
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
    async fn sends_reset_password_email_to_user_from_use_case_output() {
        let sender = Arc::new(FakeSender::default());
        let reaction = SendResetPasswordEmail::new(sender.clone());
        let user = UserDetailedOutput {
            id: "user-id".to_owned(),
            username: "alice".to_owned(),
            email: "alice@example.com".to_owned(),
        };

        reaction
            .send_reset_password_email(&user)
            .await
            .expect("reaction should send reset password email");

        assert_eq!(
            sender.sent(),
            vec![SentMail {
                to: "alice@example.com".to_owned(),
                subject: "Dentest: confirmation, password reset".to_owned(),
                plain: "Hello,\n\n\
                        Your password has been reset on Dentest!\n\n\
                        Have a nice day!"
                    .to_owned(),
            }]
        );
    }
}
