use std::env;

const DEFAULT_PORT: u16 = 3000;

pub fn server_bind_address() -> String {
    let port = configured_port();

    format!("127.0.0.1:{port}")
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

#[cfg(test)]
mod tests {
    use super::parse_port;

    #[test]
    fn parses_port() {
        assert_eq!(parse_port("8080"), 8080);
    }
}
