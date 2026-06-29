use super::{Sender, SenderError, SenderFuture, SenderResult};
use crate::infrastructure::communication::email::Mail;
use aws_config::BehaviorVersion;
use aws_sdk_sesv2::{
    Client,
    types::{Body, Content, Destination, EmailContent, Message},
};

const CHARSET: &str = "UTF-8";
const FROM_EMAIL_ADDRESS: &str = "Dentest <noreply@dentest.tech>";

pub struct SesSender {
    from_email_address: String,
}

impl SesSender {
    pub fn new() -> Self {
        let _ = dotenvy::dotenv();

        Self {
            from_email_address: FROM_EMAIL_ADDRESS.to_owned(),
        }
    }

    fn destination(to: String) -> SenderResult<Destination> {
        let to = to.trim().to_owned();

        if to.is_empty() {
            return Err(SenderError::address("recipient address must not be empty"));
        }

        Ok(Destination::builder().to_addresses(to).build())
    }

    fn content(subject: String, plain: String) -> SenderResult<EmailContent> {
        let subject = Content::builder()
            .data(subject)
            .charset(CHARSET)
            .build()
            .map_err(SenderError::message)?;
        let text = Content::builder()
            .data(plain)
            .charset(CHARSET)
            .build()
            .map_err(SenderError::message)?;
        let body = Body::builder().text(text).build();
        let message = Message::builder().subject(subject).body(body).build();

        Ok(EmailContent::builder().simple(message).build())
    }
}

impl Sender for SesSender {
    fn send<'a>(&'a self, mail: &'a dyn Mail, to: String) -> SenderFuture<'a> {
        let subject = mail.subject();
        let plain = mail.plain();

        Box::pin(async move {
            let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
            let client = Client::new(&config);

            client
                .send_email()
                .from_email_address(self.from_email_address.clone())
                .destination(Self::destination(to)?)
                .content(Self::content(subject, plain)?)
                .send()
                .await
                .map(|_| ())
                .map_err(SenderError::transport)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_plain_text_content_from_email_parts() {
        let content =
            SesSender::content("Welcome".to_owned(), "Thanks for registering.".to_owned())
                .expect("content should build");
        let message = content.simple().expect("content should be simple message");
        let subject = message.subject().expect("message should have subject");
        let body = message.body().expect("message should have body");
        let text = body.text().expect("message should have text body");

        assert_eq!(subject.data(), "Welcome");
        assert_eq!(subject.charset(), Some(CHARSET));
        assert_eq!(text.data(), "Thanks for registering.");
        assert_eq!(text.charset(), Some(CHARSET));
    }

    #[test]
    fn builds_destination_from_trimmed_recipient() {
        let destination = SesSender::destination("  alice@example.com  ".to_owned())
            .expect("destination should build");

        assert_eq!(
            destination.to_addresses(),
            &["alice@example.com".to_owned()]
        );
    }

    #[test]
    fn rejects_blank_recipient() {
        let error =
            SesSender::destination("   ".to_owned()).expect_err("blank recipient should fail");

        assert!(matches!(error, SenderError::Address(_)));
    }
}
