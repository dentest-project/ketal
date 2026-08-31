use jsonrpc_usecase::UseCaseInput;
use serde::Deserialize;

#[derive(Debug, Deserialize, UseCaseInput, PartialEq, Eq)]
pub struct RequestPasswordResetInput {
    #[transform(trim)]
    #[validate(length(min = 1, max = 255))]
    pub subject: String,
}

#[cfg(test)]
mod tests {
    use super::RequestPasswordResetInput;
    use jsonrpc_usecase::UseCaseInput;
    use serde_json::json;

    #[test]
    fn processes_request_password_reset_input_with_trimmed_subject() {
        let input: RequestPasswordResetInput = serde_json::from_value(json!({
            "subject": "  alice  ",
        }))
        .expect("request password reset input should deserialize");
        let input = input
            .into_processed()
            .expect("request password reset input should be valid");

        assert_eq!(
            input,
            RequestPasswordResetInput {
                subject: "alice".to_owned(),
            }
        );
    }

    #[test]
    fn rejects_request_password_reset_input_with_invalid_subject_length() {
        let input = serde_json::from_value::<RequestPasswordResetInput>(json!({
            "subject": "   ",
        }))
        .expect("request password reset input should deserialize");
        let errors = input
            .into_processed()
            .expect_err("blank subject should be rejected");

        assert_eq!(errors.len(), 1);
        assert_eq!(errors.violations()[0].field(), "subject");
        assert_eq!(errors.violations()[0].rule(), "length");
    }
}
