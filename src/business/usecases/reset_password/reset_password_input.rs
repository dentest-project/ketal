use jsonrpc_usecase::UseCaseInput;
use serde::Deserialize;

#[derive(Debug, Deserialize, UseCaseInput, PartialEq, Eq)]
pub struct ResetPasswordInput {
    #[transform(trim)]
    #[validate(length(min = 1, max = 50))]
    pub code: String,

    #[transform(trim)]
    #[validate(length(min = 8, max = 100))]
    pub password: String,
}

#[cfg(test)]
mod tests {
    use super::ResetPasswordInput;
    use jsonrpc_usecase::UseCaseInput;
    use serde_json::json;

    #[test]
    fn processes_reset_password_input_with_trimmed_fields() {
        let input: ResetPasswordInput = serde_json::from_value(json!({
            "code": "  reset-password-code  ",
            "password": "  new-password  ",
        }))
        .expect("reset password input should deserialize");
        let input = input
            .into_processed()
            .expect("reset password input should be valid");

        assert_eq!(
            input,
            ResetPasswordInput {
                code: "reset-password-code".to_owned(),
                password: "new-password".to_owned(),
            }
        );
    }

    #[test]
    fn rejects_reset_password_input_with_invalid_code_length() {
        let input = serde_json::from_value::<ResetPasswordInput>(json!({
            "code": "   ",
            "password": "new-password",
        }))
        .expect("reset password input should deserialize");
        let errors = input
            .into_processed()
            .expect_err("blank reset password code should be rejected");

        assert_eq!(errors.len(), 1);
        assert_eq!(errors.violations()[0].field(), "code");
        assert_eq!(errors.violations()[0].rule(), "length");
    }

    #[test]
    fn rejects_reset_password_input_with_invalid_password_length() {
        let input = serde_json::from_value::<ResetPasswordInput>(json!({
            "code": "reset-password-code",
            "password": "short",
        }))
        .expect("reset password input should deserialize");
        let errors = input
            .into_processed()
            .expect_err("short password should be rejected");

        assert_eq!(errors.len(), 1);
        assert_eq!(errors.violations()[0].field(), "password");
        assert_eq!(errors.violations()[0].rule(), "length");
    }
}
