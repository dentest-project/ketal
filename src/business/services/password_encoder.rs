use crate::business::error::PasswordEncoderError;

pub type PasswordEncoderResult<T> = Result<T, PasswordEncoderError>;

pub trait PasswordEncoder: Send + Sync {
    fn encode(&self, password: &str) -> PasswordEncoderResult<String>;
}
