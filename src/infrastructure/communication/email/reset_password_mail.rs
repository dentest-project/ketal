use super::Mail;

#[derive(Default)]
pub struct ResetPasswordMail;

impl ResetPasswordMail {
    pub fn new() -> Self {
        Self
    }
}

impl Mail for ResetPasswordMail {
    fn subject(&self) -> String {
        "Dentest: confirmation, password reset".to_owned()
    }

    fn plain(&self) -> String {
        "Hello,\n\n\
         Your password has been reset on Dentest!\n\n\
         Have a nice day!"
            .to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_reset_password_mail_subject_and_plain_text() {
        let mail = ResetPasswordMail::new();

        assert_eq!(mail.subject(), "Dentest: confirmation, password reset");
        assert_eq!(
            mail.plain(),
            "Hello,\n\n\
             Your password has been reset on Dentest!\n\n\
             Have a nice day!"
        );
    }
}
