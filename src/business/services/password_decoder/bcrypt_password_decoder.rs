use crate::business::{
    error::PasswordDecoderError,
    services::{PasswordDecoder, PasswordDecoderResult},
};

#[derive(Default)]
pub struct BcryptPasswordDecoder;

impl BcryptPasswordDecoder {
    pub fn new() -> Self {
        Self
    }
}

impl PasswordDecoder for BcryptPasswordDecoder {
    fn matches(&self, password: &str, encoded_password: &str) -> PasswordDecoderResult<bool> {
        if !is_bcrypt_hash(encoded_password) {
            return Ok(false);
        }

        bcrypt::verify(password, encoded_password)
            .map_err(|error| PasswordDecoderError::new(error.to_string()))
    }
}

fn is_bcrypt_hash(encoded_password: &str) -> bool {
    ["$2a$", "$2b$", "$2x$", "$2y$"]
        .iter()
        .any(|prefix| encoded_password.starts_with(prefix))
}

#[cfg(test)]
mod tests {
    use super::BcryptPasswordDecoder;
    use crate::business::services::PasswordDecoder;

    const TEST_BCRYPT_COST: u32 = 4;

    #[test]
    fn matches_bcrypt_password() {
        let encoded_password =
            bcrypt::hash("secret123", TEST_BCRYPT_COST).expect("password should hash");

        assert!(
            BcryptPasswordDecoder::new()
                .matches("secret123", &encoded_password)
                .expect("password should be decoded")
        );
    }

    #[test]
    fn matches_legacy_bcrypt_password() {
        let encoded_password = bcrypt::hash("secret123", TEST_BCRYPT_COST)
            .expect("password should hash")
            .replacen("$2b$", "$2y$", 1);

        assert!(
            BcryptPasswordDecoder::new()
                .matches("secret123", &encoded_password)
                .expect("password should be decoded")
        );
    }

    #[test]
    fn returns_false_when_bcrypt_password_does_not_match() {
        let encoded_password =
            bcrypt::hash("secret123", TEST_BCRYPT_COST).expect("password should hash");

        assert!(
            !BcryptPasswordDecoder::new()
                .matches("wrong-password", &encoded_password)
                .expect("password should be decoded")
        );
    }

    #[test]
    fn returns_false_for_unsupported_hash_format() {
        assert!(
            !BcryptPasswordDecoder::new()
                .matches("secret123", "$argon2id$v=19$m=19456,t=2,p=1$salt$hash")
                .expect("unsupported hashes should not fail")
        );
    }
}
