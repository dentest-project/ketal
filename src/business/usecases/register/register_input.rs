use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(try_from = "RawRegisterInput")]
pub struct RegisterInput {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
struct RawRegisterInput {
    username: String,
    email: String,
    password: String,
}

impl TryFrom<RawRegisterInput> for RegisterInput {
    type Error = String;

    fn try_from(input: RawRegisterInput) -> Result<Self, Self::Error> {
        Ok(Self {
            username: trim_and_validate(input.username, "username", 1, 50)?,
            email: trim_and_validate(input.email, "email", 1, 255)?,
            password: trim_and_validate(input.password, "password", 8, 64)?,
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
    use super::RegisterInput;
    use serde_json::json;

    #[test]
    fn deserializes_register_input_with_trimmed_fields() {
        let input: RegisterInput = serde_json::from_value(json!({
            "username": "  alice  ",
            "email": "  alice@example.com  ",
            "password": "  secret123  "
        }))
        .expect("register input should deserialize");

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
        let error = serde_json::from_value::<RegisterInput>(json!({
            "username": "   ",
            "email": "alice@example.com",
            "password": "secret123"
        }))
        .expect_err("blank username should be rejected");

        assert!(
            error
                .to_string()
                .contains("username must be between 1 and 50 characters")
        );
    }
}
