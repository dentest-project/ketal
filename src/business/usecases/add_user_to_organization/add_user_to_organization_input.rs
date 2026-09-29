use jsonrpc_usecase::UseCaseInput;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize, UseCaseInput, PartialEq, Eq)]
pub struct AddUserToOrganizationInput {
    pub organization_id: Uuid,
    pub user_id: Uuid,
}

#[cfg(test)]
mod tests {
    use super::AddUserToOrganizationInput;
    use jsonrpc_usecase::UseCaseInput;
    use serde_json::json;
    use std::error::Error;
    use uuid::Uuid;

    #[test]
    fn accepts_uuid_identifiers() -> Result<(), Box<dyn Error>> {
        let organization_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let input: AddUserToOrganizationInput = serde_json::from_value(json!({
            "organization_id": organization_id,
            "user_id": user_id,
        }))?;

        assert_eq!(
            input.into_processed()?,
            AddUserToOrganizationInput {
                organization_id,
                user_id
            }
        );
        Ok(())
    }

    #[test]
    fn rejects_missing_and_invalid_identifiers() {
        let id = Uuid::new_v4();
        for value in [
            json!({}),
            json!({"organization_id": id}),
            json!({"user_id": id}),
            json!({"organization_id": "invalid", "user_id": id}),
            json!({"organization_id": id, "user_id": "invalid"}),
            json!({"organization_id": null, "user_id": id}),
            json!({"organization_id": id, "user_id": 42}),
        ] {
            assert!(serde_json::from_value::<AddUserToOrganizationInput>(value).is_err());
        }
    }
}
