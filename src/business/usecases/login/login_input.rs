use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(try_from = "RawLoginInput")]
pub struct LoginInput {
    pub subject: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawLoginInput {
    subject: String,
    password: String,
}

impl TryFrom<RawLoginInput> for LoginInput {
    type Error = String;

    fn try_from(input: RawLoginInput) -> Result<Self, Self::Error> {
        Ok(Self {
            subject: trim_and_validate(input.subject, "subject", 1, 255)?,
            password: trim_and_validate(input.password, "password", 1, 64)?,
        })
    }
}

fn trim_and_validate(
    value: String,
    field_name: &str,
    min_len: usize,
    max_len: usize,
) -> Result<String, String> {
    let trimmed_value = value.trim().to_owned();
    let length = trimmed_value.chars().count();

    if !(min_len..=max_len).contains(&length) {
        return Err(format!(
            "{field_name} must be between {min_len} and {max_len} characters"
        ));
    }

    Ok(trimmed_value)
}

#[cfg(test)]
mod tests {
    use super::LoginInput;
    use serde_json::json;

    #[test]
    fn deserializes_login_input_with_trimmed_fields() {
        let input: LoginInput = serde_json::from_value(json!({
            "subject": "  alice  ",
            "password": "  secret123  "
        }))
        .expect("login input should deserialize");

        assert_eq!(
            input,
            LoginInput {
                subject: "alice".to_owned(),
                password: "secret123".to_owned(),
            }
        );
    }

    #[test]
    fn rejects_login_input_with_invalid_length() {
        let error = serde_json::from_value::<LoginInput>(json!({
            "subject": "   ",
            "password": "secret123"
        }))
        .expect_err("blank subject should be rejected");

        assert!(
            error
                .to_string()
                .contains("subject must be between 1 and 255 characters")
        );
    }
}
