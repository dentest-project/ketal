use super::ResetPasswordCodeGenerator;
use uuid::Uuid;

#[derive(Default)]
pub struct UuidResetPasswordCodeGenerator;

impl UuidResetPasswordCodeGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl ResetPasswordCodeGenerator for UuidResetPasswordCodeGenerator {
    fn generate(&self) -> String {
        Uuid::new_v4().simple().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_uuid_v4_without_dashes() {
        let code = UuidResetPasswordCodeGenerator::new().generate();

        assert_eq!(code.len(), 32);
        assert!(!code.contains('-'));
        assert_eq!(&code[12..13], "4");
    }
}
