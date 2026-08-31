use jsonrpc_usecase::UseCaseInput;
use serde::Deserialize;

#[derive(Debug, Deserialize, UseCaseInput, PartialEq, Eq)]
pub struct LoginInput {
    #[transform(trim)]
    #[validate(length(min = 1, max = 255))]
    pub subject: String,

    #[transform(trim)]
    #[validate(length(min = 1, max = 64))]
    pub password: String,
}

#[cfg(test)]
mod tests {
    use super::LoginInput;
    use jsonrpc_usecase::UseCaseInput;
    use serde_json::json;

    #[test]
    fn processes_login_input_with_trimmed_fields() {
        let input: LoginInput = serde_json::from_value(json!({
            "subject": "  alice  ",
            "password": "  secret123  "
        }))
        .expect("login input should deserialize");
        let input = input.into_processed().expect("login input should be valid");

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
        let input = serde_json::from_value::<LoginInput>(json!({
            "subject": "   ",
            "password": "secret123"
        }))
        .expect("login input should deserialize");
        let errors = input
            .into_processed()
            .expect_err("blank subject should be rejected");

        assert_eq!(errors.len(), 1);
        assert_eq!(errors.violations()[0].field(), "subject");
        assert_eq!(errors.violations()[0].rule(), "length");
    }
}
