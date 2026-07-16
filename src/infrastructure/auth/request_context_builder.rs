use super::jwt_token_decoder::JwtTokenDecoder;
use crate::{
    business::{RequestContext, entities::user::user_gateway::UserGateway},
    infrastructure::config::jwt_secrets,
};
use jsonrpc_usecase::{ContextBuilderRequest, RequestHeaders};
use std::sync::Arc;

#[derive(Clone)]
pub struct RequestContextBuilder {
    user_gateway: Arc<dyn UserGateway>,
    token_decoder: Arc<JwtTokenDecoder>,
}

impl RequestContextBuilder {
    pub fn new(user_gateway: Arc<dyn UserGateway>) -> Self {
        let jwt_secrets = jwt_secrets();
        let token_decoder = JwtTokenDecoder::from_public_key(jwt_secrets.public_key())
            .expect("JWT_PUBLIC_KEY should contain a usable RSA PEM public key");

        Self {
            user_gateway,
            token_decoder: Arc::new(token_decoder),
        }
    }

    pub async fn build(&self, request: ContextBuilderRequest) -> RequestContext {
        self.build_from_headers(request.headers()).await
    }

    async fn build_from_headers(&self, headers: &RequestHeaders) -> RequestContext {
        let user = match bearer_token(headers) {
            Some(token) => self.user_from_token(token).await,
            None => None,
        };

        RequestContext { user }
    }

    async fn user_from_token(&self, token: &str) -> Option<crate::business::entities::user::User> {
        let username = self.token_decoder.decode_username(token).ok()?;

        self.user_gateway
            .find_one_by_username(&username)
            .await
            .ok()
            .flatten()
    }
}

fn bearer_token(headers: &RequestHeaders) -> Option<&str> {
    let authorization = headers.get("authorization")?;
    let mut parts = authorization.split_whitespace();
    let scheme = parts.next()?;
    let token = parts.next()?;

    if parts.next().is_some() || !scheme.eq_ignore_ascii_case("bearer") {
        return None;
    }

    Some(token)
}

#[cfg(test)]
mod tests {
    use super::{RequestContextBuilder, bearer_token};
    use crate::business::{
        EntityBuilder,
        entities::user::{
            UserBuilder,
            user_gateway::{InMemoryUserGateway, UserGateway},
        },
    };
    use jsonrpc_usecase::RequestHeaders;
    use jsonwebtoken::{
        Algorithm, DecodingKey, EncodingKey, Header, encode, get_current_timestamp,
    };
    use serde::Serialize;
    use std::sync::Arc;

    use super::super::jwt_token_decoder::JwtTokenDecoder;

    const TEST_SECRET: &[u8] = b"test-secret";

    #[derive(Serialize)]
    struct TestClaims {
        username: String,
        exp: u64,
    }

    #[tokio::test]
    async fn injects_user_from_bearer_token() {
        let user_gateway = Arc::new(InMemoryUserGateway::default());
        let user = UserBuilder::init()
            .with_username("alice".to_owned())
            .with_email("alice@example.com".to_owned())
            .with_password("hashed-password".to_owned())
            .build();
        user_gateway
            .save(&user)
            .await
            .expect("user should be saved");
        let token = token_for_username(user.username());
        let builder = request_context_builder(user_gateway);
        let headers = RequestHeaders::new([("authorization", format!("Bearer {token}"))]);

        let context = builder.build_from_headers(&headers).await;

        assert_eq!(context.user, Some(user));
    }

    #[tokio::test]
    async fn leaves_user_empty_without_bearer_token() {
        let builder = request_context_builder(Arc::new(InMemoryUserGateway::default()));
        let headers = RequestHeaders::default();

        let context = builder.build_from_headers(&headers).await;

        assert!(context.user.is_none());
    }

    #[tokio::test]
    async fn leaves_user_empty_when_token_is_invalid() {
        let builder = request_context_builder(Arc::new(InMemoryUserGateway::default()));
        let headers = RequestHeaders::new([("authorization", "Bearer invalid-token")]);

        let context = builder.build_from_headers(&headers).await;

        assert!(context.user.is_none());
    }

    #[test]
    fn reads_bearer_token_from_authorization_header() {
        let headers = RequestHeaders::new([("Authorization", "Bearer token")]);

        assert_eq!(bearer_token(&headers), Some("token"));
    }

    #[test]
    fn rejects_non_bearer_authorization_header() {
        let headers = RequestHeaders::new([("authorization", "Basic token")]);

        assert_eq!(bearer_token(&headers), None);
    }

    fn request_context_builder(user_gateway: Arc<dyn UserGateway>) -> RequestContextBuilder {
        RequestContextBuilder {
            user_gateway,
            token_decoder: Arc::new(test_decoder()),
        }
    }

    fn test_decoder() -> JwtTokenDecoder {
        JwtTokenDecoder::from_decoding_key(DecodingKey::from_secret(TEST_SECRET), Algorithm::HS256)
    }

    fn token_for_username(username: impl Into<String>) -> String {
        let claims = TestClaims {
            username: username.into(),
            exp: get_current_timestamp() + 3600,
        };

        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(TEST_SECRET),
        )
        .expect("test token should be encoded")
    }
}
