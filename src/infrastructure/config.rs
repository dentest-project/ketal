use std::env;

use axum::http::HeaderValue;

const DEFAULT_PORT: u16 = 3000;
const CORS_ALLOWED_ORIGIN: &str = "CORS_ALLOWED_ORIGIN";

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
