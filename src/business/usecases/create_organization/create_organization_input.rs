use jsonrpc_usecase::UseCaseInput;
use serde::Deserialize;

#[derive(Debug, Deserialize, UseCaseInput, PartialEq, Eq)]
pub struct CreateOrganizationInput {
    #[transform(trim)]
    #[validate(length(min = 1, max = 255))]
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::CreateOrganizationInput;
    use jsonrpc_usecase::UseCaseInput;
    use serde_json::json;
    use std::error::Error;

    #[test]
    fn trims_names_and_accepts_length_boundaries() -> Result<(), Box<dyn Error>> {
        for name in ["A".to_owned(), "É".repeat(255)] {
            let input: CreateOrganizationInput =
                serde_json::from_value(json!({ "name": format!("  {name}  ") }))?;

            assert_eq!(input.into_processed()?.name, name);
        }

        Ok(())
    }

    #[test]
    fn rejects_empty_blank_and_overlong_names() -> Result<(), Box<dyn Error>> {
        for name in [String::new(), " \t\n ".to_owned(), "É".repeat(256)] {
            let input: CreateOrganizationInput = serde_json::from_value(json!({ "name": name }))?;
            let result = input.into_processed();

            assert!(result.is_err());
            if let Err(errors) = result {
                assert_eq!(errors.len(), 1);
                assert!(errors.violations().iter().all(|violation| {
                    violation.field() == "name" && violation.rule() == "length"
                }));
            }
        }

        Ok(())
    }

    #[test]
    fn requires_a_string_name() {
        for value in [json!({}), json!({ "name": null }), json!({ "name": 42 })] {
            assert!(serde_json::from_value::<CreateOrganizationInput>(value).is_err());
        }
    }
}
