use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(try_from = "RawRequestPasswordResetInput")]
pub struct RequestPasswordResetInput {
    pub subject: String,
}

#[derive(Debug, Deserialize)]
struct RawRequestPasswordResetInput {
    subject: String,
}

impl TryFrom<RawRequestPasswordResetInput> for RequestPasswordResetInput {
    type Error = String;

    fn try_from(input: RawRequestPasswordResetInput) -> Result<Self, Self::Error> {
        Ok(Self {
            subject: trim_and_validate(input.subject, "subject", 1, 255)?,
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
    use super::RequestPasswordResetInput;
    use serde_json::json;

    #[test]
    fn deserializes_request_password_reset_input_with_trimmed_subject() {
        let input: RequestPasswordResetInput = serde_json::from_value(json!({
            "subject": "  alice  ",
        }))
        .expect("request password reset input should deserialize");

        assert_eq!(
            input,
            RequestPasswordResetInput {
                subject: "alice".to_owned(),
            }
        );
    }

    #[test]
    fn rejects_request_password_reset_input_with_invalid_subject_length() {
        let error = serde_json::from_value::<RequestPasswordResetInput>(json!({
            "subject": "   ",
        }))
        .expect_err("blank subject should be rejected");

        assert!(
            error
                .to_string()
                .contains("subject must be between 1 and 255 characters")
        );
    }
}
