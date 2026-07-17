use crate::{
    business::{
        entities::user::User,
        error::TokenGeneratorError,
        services::{TokenGenerator, TokenGeneratorResult},
    },
    infrastructure::config::jwt_secrets,
};
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode, get_current_timestamp};
use rsa::{RsaPrivateKey, pkcs1::EncodeRsaPrivateKey, pkcs8::DecodePrivateKey};
use serde::Serialize;

const TOKEN_LIFETIME_SECONDS: u64 = 14 * 24 * 60 * 60;

pub struct JwtTokenGenerator {
    encoding_key: EncodingKey,
}

impl JwtTokenGenerator {
    pub fn new() -> Self {
        let jwt_secrets = jwt_secrets();
        let encoding_key = encoding_key_from_private_key(
            jwt_secrets.secret_key(),
            jwt_secrets.passphrase().as_bytes(),
        )
        .expect("JWT_SECRET_KEY should contain a usable encrypted RSA PEM private key");

        Self { encoding_key }
    }
}

impl TokenGenerator for JwtTokenGenerator {
    fn generate_for(&self, user: &User) -> TokenGeneratorResult<String> {
        let issued_at = get_current_timestamp();
        let claims = JwtClaims {
            iat: issued_at,
            exp: issued_at + TOKEN_LIFETIME_SECONDS,
            roles: vec!["ROLE_USER"],
            username: user.username(),
        };

        encode(&Header::new(Algorithm::RS256), &claims, &self.encoding_key)
            .map_err(|error| TokenGeneratorError::new(error.to_string()))
    }
}

#[derive(Serialize)]
struct JwtClaims<'a> {
    iat: u64,
    exp: u64,
    roles: Vec<&'static str>,
    username: &'a str,
}

fn encoding_key_from_private_key(
    private_key: &[u8],
    passphrase: &[u8],
) -> Result<EncodingKey, TokenGeneratorError> {
    private_key_from_pem(private_key, passphrase)
        .and_then(|private_key| {
            private_key
                .to_pkcs1_der()
                .map_err(|error| TokenGeneratorError::new(error.to_string()))
        })
        .map(|private_key| EncodingKey::from_rsa_der(private_key.as_bytes()))
}

fn private_key_from_pem(
    private_key: &[u8],
    passphrase: &[u8],
) -> Result<RsaPrivateKey, TokenGeneratorError> {
    let private_key = std::str::from_utf8(private_key)
        .map_err(|error| TokenGeneratorError::new(error.to_string()))?;

    RsaPrivateKey::from_pkcs8_encrypted_pem(private_key, passphrase)
        .map_err(|error| TokenGeneratorError::new(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{JwtTokenGenerator, TOKEN_LIFETIME_SECONDS, encoding_key_from_private_key};
    use crate::business::{
        EntityBuilder,
        entities::user::UserBuilder,
        services::{TokenGenerator, TokenGeneratorResult},
    };
    use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
    use rand_core::OsRng;
    use rsa::{
        RsaPrivateKey, RsaPublicKey,
        pkcs1::EncodeRsaPublicKey,
        pkcs8::{EncodePrivateKey, LineEnding},
    };
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct TestClaims {
        iat: u64,
        exp: u64,
        roles: Vec<String>,
        username: String,
    }

    #[test]
    fn generates_valid_rs256_jwt_from_encrypted_private_key() {
        let mut rng = OsRng;
        let private_key =
            RsaPrivateKey::new(&mut rng, 2048).expect("test private key should be generated");
        let encrypted_private_key = private_key
            .to_pkcs8_encrypted_pem(&mut rng, "test-passphrase", LineEnding::LF)
            .expect("test private key should be encrypted");
        let public_key = RsaPublicKey::from(&private_key)
            .to_pkcs1_der()
            .expect("test public key should be encoded");
        let encoding_key =
            encoding_key_from_private_key(encrypted_private_key.as_bytes(), b"test-passphrase")
                .expect("encrypted private key should create an encoding key");
        let token_generator = JwtTokenGenerator { encoding_key };
        let user = UserBuilder::init()
            .with_username("pierre".to_owned())
            .with_email("pierre@example.com".to_owned())
            .with_password("hashed-password".to_owned())
            .build();

        let token = token_generator
            .generate_for(&user)
            .expect("token should be generated");

        let claims = decode_token(&token, public_key.as_bytes())
            .expect("generated token should pass RS256 validation");
        assert_eq!(claims.username, "pierre");
        assert_eq!(claims.roles, vec!["ROLE_USER"]);
        assert_eq!(claims.exp - claims.iat, TOKEN_LIFETIME_SECONDS);
    }

    #[test]
    fn rejects_encrypted_private_key_when_passphrase_is_invalid() {
        let mut rng = OsRng;
        let private_key =
            RsaPrivateKey::new(&mut rng, 2048).expect("test private key should be generated");
        let encrypted_private_key = private_key
            .to_pkcs8_encrypted_pem(&mut rng, "test-passphrase", LineEnding::LF)
            .expect("test private key should be encrypted");

        let error = encoding_key_from_private_key(encrypted_private_key.as_bytes(), b"wrong")
            .expect_err("invalid passphrase should be rejected");

        assert!(!error.to_string().is_empty());
    }

    fn decode_token(token: &str, public_key: &[u8]) -> TokenGeneratorResult<TestClaims> {
        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_aud = false;

        decode::<TestClaims>(token, &DecodingKey::from_rsa_der(public_key), &validation)
            .map(|token| token.claims)
            .map_err(|error| crate::business::error::TokenGeneratorError::new(error.to_string()))
    }
}
