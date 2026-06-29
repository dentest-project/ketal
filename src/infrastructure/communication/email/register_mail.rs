use super::Mail;

pub struct RegisterMail {
    username: String,
}

impl RegisterMail {
    pub fn new(username: String) -> Self {
        Self { username }
    }
}

impl Mail for RegisterMail {
    fn subject(&self) -> String {
        "Welcome on Dentest!".to_owned()
    }

    fn plain(&self) -> String {
        format!(
            "Welcome on Dentest!\n\n\
             You are now a user of a platform on which you'll be able to write, read, pull and execute Gherkin features, in order to specify, validate and document your application.\n\n\
             Your username is: {}\n\n\
             Enjoy ;)",
            self.username
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_register_mail_subject_and_plain_text() {
        let mail = RegisterMail::new("alice".to_owned());

        assert_eq!(mail.subject(), "Welcome on Dentest!");
        assert_eq!(
            mail.plain(),
            "Welcome on Dentest!\n\n\
             You are now a user of a platform on which you'll be able to write, read, pull and execute Gherkin features, in order to specify, validate and document your application.\n\n\
             Your username is: alice\n\n\
             Enjoy ;)"
        );
    }
}
