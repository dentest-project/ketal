#[cfg(test)]
pub mod reset_password_code_generator_double;
mod uuid_reset_password_code_generator;

#[cfg(test)]
pub(crate) use reset_password_code_generator_double::ResetPasswordCodeGeneratorDouble;
pub use uuid_reset_password_code_generator::UuidResetPasswordCodeGenerator;

pub trait ResetPasswordCodeGenerator: Send + Sync {
    fn generate(&self) -> String;
}
