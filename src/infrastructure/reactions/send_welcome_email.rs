mod default_send_welcome_email;

use crate::{
    business::usecases::outputs::user::UserDetailedOutput,
    infrastructure::communication::email::{RegisterMail, Sender, SenderError},
};
use jsonrpc_usecase::{UseCaseEvent, UseCaseEventConsumer};
use std::sync::Arc;

pub struct SendWelcomeEmail {
    sender: Arc<dyn Sender>,
}

impl SendWelcomeEmail {
    pub fn new(sender: Arc<dyn Sender>) -> Self {
        Self { sender }
    }

    async fn send_welcome_email(&self, user: &UserDetailedOutput) -> Result<(), SenderError> {
        let mail = RegisterMail::new(user.username.clone());

        self.sender.send(&mail, user.email.clone()).await
    }
}

#[UseCaseEventConsumer(event = "DidRegister")]
impl SendWelcomeEmail {
    async fn consume(&self, event: &UseCaseEvent) {
        let Some(user) = event.get_output::<UserDetailedOutput>() else {
            eprintln!("DidRegister event output is missing or has an unexpected type");
            return;
        };

        if let Err(error) = self.send_welcome_email(user).await {
            eprintln!("failed to send welcome email: {error}");
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
    async fn sends_welcome_email_to_registered_user() {
        let sender = Arc::new(FakeSender::default());
        let reaction = SendWelcomeEmail::new(sender.clone());
        let user = UserDetailedOutput {
            id: "user-id".to_owned(),
            username: "alice".to_owned(),
            email: "alice@example.com".to_owned(),
        };

        reaction
            .send_welcome_email(&user)
            .await
            .expect("reaction should send welcome email");

        assert_eq!(
            sender.sent(),
            vec![SentMail {
                to: "alice@example.com".to_owned(),
                subject: "Welcome on Dentest!".to_owned(),
                plain: "Welcome on Dentest!\n\n\
                        You are now a user of a platform on which you'll be able to write, read, pull and execute Gherkin features, in order to specify, validate and document your application.\n\n\
                        Your username is: alice\n\n\
                        Enjoy ;)"
                    .to_owned(),
            }]
        );
    }
}
