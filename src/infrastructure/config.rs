use std::{env, fs};

use axum::http::HeaderValue;

const DEFAULT_PORT: u16 = 3000;
const CORS_ALLOWED_ORIGIN: &str = "CORS_ALLOWED_ORIGIN";
const JWT_SECRET_KEY: &str = "JWT_SECRET_KEY";
const JWT_PUBLIC_KEY: &str = "JWT_PUBLIC_KEY";
const JWT_PASSPHRASE: &str = "JWT_PASSPHRASE";

pub struct JwtSecrets {
    secret_key: Vec<u8>,
    public_key: Vec<u8>,
    passphrase: String,
}

impl JwtSecrets {
    pub fn secret_key(&self) -> &[u8] {
        &self.secret_key
    }

    pub fn public_key(&self) -> &[u8] {
        &self.public_key
    }

    pub fn passphrase(&self) -> &str {
        &self.passphrase
    }
}

pub fn server_bind_address() -> String {
    let port = configured_port();

    format!("127.0.0.1:{port}")
}

pub fn cors_allowed_origin() -> HeaderValue {
    let _ = dotenvy::dotenv();

    match env::var(CORS_ALLOWED_ORIGIN) {
        Ok(origin) => parse_header_value(CORS_ALLOWED_ORIGIN, &origin),
        Err(env::VarError::NotPresent) => {
            panic!("{CORS_ALLOWED_ORIGIN} must be set to the allowed browser origin")
        }
        Err(error) => panic!("{CORS_ALLOWED_ORIGIN} must be valid Unicode: {error}"),
    }
}

pub fn jwt_secrets() -> JwtSecrets {
    let _ = dotenvy::dotenv();

    let secret_key = read_secret_file(JWT_SECRET_KEY);
    let public_key = read_secret_file(JWT_PUBLIC_KEY);
    let passphrase = required_env(JWT_PASSPHRASE, "the JWT private key passphrase");

    if passphrase.is_empty() {
        panic!("{JWT_PASSPHRASE} must not be empty");
    }

    JwtSecrets {
        secret_key,
        public_key,
        passphrase,
    }
}

fn configured_port() -> u16 {
    let _ = dotenvy::dotenv();

    match env::var("PORT") {
        Ok(port) => parse_port(&port),
        Err(env::VarError::NotPresent) => DEFAULT_PORT,
        Err(error) => panic!("PORT must be valid Unicode: {error}"),
    }
}

fn parse_port(port: &str) -> u16 {
    port.parse()
        .unwrap_or_else(|error| panic!("PORT must be a valid TCP port (0-65535): {error}"))
}

fn parse_header_value(name: &str, value: &str) -> HeaderValue {
    value
        .parse()
        .unwrap_or_else(|error| panic!("{name} must be a valid HTTP header value: {error}"))
}

fn read_secret_file(env_name: &str) -> Vec<u8> {
    let path = required_env(env_name, "a readable file path");
    let content = fs::read(&path).unwrap_or_else(|error| {
        panic!("failed to read file configured at {env_name} ({path}): {error}")
    });

    if content.is_empty() {
        panic!("file configured at {env_name} ({path}) must not be empty");
    }

    content
}

fn required_env(name: &str, description: &str) -> String {
    match env::var(name) {
        Ok(value) => value,
        Err(env::VarError::NotPresent) => panic!("{name} must be set to {description}"),
        Err(error) => panic!("{name} must be valid Unicode: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_header_value, parse_port};

    #[test]
    fn parses_port() {
        assert_eq!(parse_port("8080"), 8080);
    }

    #[test]
    fn parses_header_value() {
        assert_eq!(
            parse_header_value("HEADER", "http://localhost:5173"),
            "http://localhost:5173"
        );
    }
}
