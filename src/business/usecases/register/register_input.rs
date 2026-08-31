use jsonrpc_usecase::UseCaseInput;
use serde::Deserialize;

#[derive(Debug, Deserialize, UseCaseInput, PartialEq, Eq)]
pub struct RegisterInput {
    #[transform(trim)]
    #[validate(length(min = 1, max = 50))]
    pub username: String,

    #[transform(trim)]
    #[validate(length(min = 1, max = 255))]
    pub email: String,

    #[transform(trim)]
    #[validate(length(min = 8, max = 64))]
    pub password: String,
}

#[cfg(test)]
mod tests {
    use super::RegisterInput;
    use jsonrpc_usecase::UseCaseInput;
    use serde_json::json;

    #[test]
    fn processes_register_input_with_trimmed_fields() {
        let input: RegisterInput = serde_json::from_value(json!({
            "username": "  alice  ",
            "email": "  alice@example.com  ",
            "password": "  secret123  "
        }))
        .expect("register input should deserialize");
        let input = input
            .into_processed()
            .expect("register input should be valid");

        assert_eq!(
            input,
            RegisterInput {
                username: "alice".to_owned(),
                email: "alice@example.com".to_owned(),
                password: "secret123".to_owned(),
            }
        );
    }

    #[test]
    fn rejects_register_input_with_invalid_length() {
        let input = serde_json::from_value::<RegisterInput>(json!({
            "username": "   ",
            "email": "alice@example.com",
            "password": "secret123"
        }))
        .expect("register input should deserialize");
        let errors = input
            .into_processed()
            .expect_err("blank username should be rejected");

        assert_eq!(errors.len(), 1);
        assert_eq!(errors.violations()[0].field(), "username");
        assert_eq!(errors.violations()[0].rule(), "length");
    }
}
