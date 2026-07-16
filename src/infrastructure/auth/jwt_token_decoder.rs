use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::Deserialize;
use thiserror::Error;

#[derive(Clone)]
pub struct JwtTokenDecoder {
    decoding_key: DecodingKey,
    validation: Validation,
}

impl JwtTokenDecoder {
    pub fn from_public_key(public_key: &[u8]) -> Result<Self, JwtTokenDecoderError> {
        let decoding_key = DecodingKey::from_rsa_pem(public_key)
            .map_err(JwtTokenDecoderError::InvalidDecodingKey)?;

        Ok(Self::from_decoding_key(decoding_key, Algorithm::RS256))
    }

    pub fn decode_username(&self, token: &str) -> Result<String, JwtTokenDecoderError> {
        let token = decode::<JwtClaims>(token, &self.decoding_key, &self.validation)
            .map_err(JwtTokenDecoderError::InvalidToken)?;
        let username = token.claims.username.trim();

        if username.is_empty() {
            return Err(JwtTokenDecoderError::InvalidUsername);
        }

        Ok(username.to_owned())
    }

    pub(crate) fn from_decoding_key(decoding_key: DecodingKey, algorithm: Algorithm) -> Self {
        let mut validation = Validation::new(algorithm);
        // Ketal does not configure an expected audience yet; signature and exp stay enforced.
        validation.validate_aud = false;

        Self {
            decoding_key,
            validation,
        }
    }
}

#[derive(Debug, Error)]
pub enum JwtTokenDecoderError {
    #[error("JWT public key should be usable for token decoding: {0}")]
    InvalidDecodingKey(jsonwebtoken::errors::Error),
    #[error("JWT should be valid: {0}")]
    InvalidToken(jsonwebtoken::errors::Error),
    #[error("JWT username should not be blank")]
    InvalidUsername,
}

#[derive(Debug, Deserialize)]
struct JwtClaims {
    username: String,
}

#[cfg(test)]
mod tests {
    use super::JwtTokenDecoder;
    use jsonwebtoken::{
        Algorithm, DecodingKey, EncodingKey, Header, encode, get_current_timestamp,
    };
    use serde::Serialize;

    const TEST_SECRET: &[u8] = b"test-secret";
    const OTHER_TEST_SECRET: &[u8] = b"other-test-secret";

    #[derive(Serialize)]
    struct TestClaims {
        username: String,
        exp: u64,
    }

    #[test]
    fn decodes_username_from_valid_token() {
        let decoder = JwtTokenDecoder::from_decoding_key(
            DecodingKey::from_secret(TEST_SECRET),
            Algorithm::HS256,
        );
        let token = token_for_username(TEST_SECRET, "pierre", get_current_timestamp() + 3600);

        let username = decoder
            .decode_username(&token)
            .expect("token should decode to a username");

        assert_eq!(username, "pierre");
    }

    #[test]
    fn rejects_expired_token() {
        let decoder = JwtTokenDecoder::from_decoding_key(
            DecodingKey::from_secret(TEST_SECRET),
            Algorithm::HS256,
        );
        let token = token_for_username(TEST_SECRET, "pierre", get_current_timestamp() - 3600);

        assert!(
            decoder.decode_username(&token).is_err(),
            "expired tokens should be rejected"
        );
    }

    #[test]
    fn rejects_token_with_invalid_signature() {
        let decoder = JwtTokenDecoder::from_decoding_key(
            DecodingKey::from_secret(TEST_SECRET),
            Algorithm::HS256,
        );
        let token = token_for_username(OTHER_TEST_SECRET, "pierre", get_current_timestamp() + 3600);

        assert!(
            decoder.decode_username(&token).is_err(),
            "tokens signed with another key should be rejected"
        );
    }

    fn token_for_username(secret: &[u8], username: impl Into<String>, exp: u64) -> String {
        let claims = TestClaims {
            username: username.into(),
            exp,
        };

        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(secret),
        )
        .expect("test token should be encoded")
    }
}
