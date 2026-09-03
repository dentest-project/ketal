use jsonrpc_usecase::UseCaseInput;
use serde::Deserialize;

#[derive(Debug, Deserialize, UseCaseInput, PartialEq, Eq)]
pub struct UpdateMyPersonalInformationInput {
    #[transform(trim)]
    #[validate(length(min = 1, max = 50))]
    pub username: String,

    #[transform(trim)]
    #[validate(length(min = 1, max = 255))]
    pub email: String,

    #[transform(trim)]
    #[validate(length(min = 8, max = 64))]
    pub password: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::UpdateMyPersonalInformationInput;
    use jsonrpc_usecase::UseCaseInput;
    use serde_json::json;
    use std::{error::Error, io};

    #[test]
    fn processes_trimmed_fields_and_an_optional_password() -> Result<(), Box<dyn Error>> {
        let input = serde_json::from_value::<UpdateMyPersonalInformationInput>(json!({
            "username": "  alice  ",
            "email": "  alice@example.com  ",
            "password": "  secret123  "
        }))?
        .into_processed()?;

        assert_eq!(
            input,
            UpdateMyPersonalInformationInput {
                username: "alice".to_owned(),
                email: "alice@example.com".to_owned(),
                password: Some("secret123".to_owned()),
            }
        );

        Ok(())
    }

    #[test]
    fn accepts_an_omitted_or_null_password() -> Result<(), Box<dyn Error>> {
        for value in [
            json!({
                "username": "alice",
                "email": "alice@example.com"
            }),
            json!({
                "username": "alice",
                "email": "alice@example.com",
                "password": null
            }),
        ] {
            let input = serde_json::from_value::<UpdateMyPersonalInformationInput>(value)?
                .into_processed()?;

            assert_eq!(input.password, None);
        }

        Ok(())
    }

    #[test]
    fn rejects_a_provided_password_with_an_invalid_length() -> Result<(), Box<dyn Error>> {
        let input = serde_json::from_value::<UpdateMyPersonalInformationInput>(json!({
            "username": "alice",
            "email": "alice@example.com",
            "password": " short "
        }))?;

        let errors = match input.into_processed() {
            Err(errors) => errors,
            Ok(_) => return Err(io::Error::other("a short password should be rejected").into()),
        };
        assert_eq!(errors.len(), 1);
        let violation = errors
            .violations()
            .first()
            .ok_or_else(|| io::Error::other("a password violation should be returned"))?;
        assert_eq!(violation.field(), "password");
        assert_eq!(violation.rule(), "length");

        Ok(())
    }
}
