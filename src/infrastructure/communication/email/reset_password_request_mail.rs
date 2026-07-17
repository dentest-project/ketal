use super::Mail;

pub struct ResetPasswordRequestMail {
    reset_password_link: String,
}

impl ResetPasswordRequestMail {
    pub fn new(reset_password_link: String) -> Self {
        Self {
            reset_password_link,
        }
    }
}

impl Mail for ResetPasswordRequestMail {
    fn subject(&self) -> String {
        "Dentest: reset your password".to_owned()
    }

    fn plain(&self) -> String {
        format!(
            "Hello,\n\n\
             Click on the following link in order to reset your password: {}\n\n\
             Have a nice day!",
            self.reset_password_link
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_reset_password_request_mail_subject_and_plain_text() {
        let mail = ResetPasswordRequestMail::new(
            "http://localhost:5173/reset-password?code=reset-password-code".to_owned(),
        );

        assert_eq!(mail.subject(), "Dentest: reset your password");
        assert_eq!(
            mail.plain(),
            "Hello,\n\n\
             Click on the following link in order to reset your password: http://localhost:5173/reset-password?code=reset-password-code\n\n\
             Have a nice day!"
        );
    }
}
