mod database;

pub mod business;
pub mod infrastructure;

use jsonrpc_usecase::{JsonRpcService, RegistrationError};
use std::sync::Arc;

use crate::{
    business::entities::user::user_gateway::SqlxUserGateway,
    infrastructure::auth::RequestContextBuilder,
};

pub fn build_service() -> Result<JsonRpcService, RegistrationError> {
    let context_builder = RequestContextBuilder::new(Arc::new(SqlxUserGateway::new()));

    JsonRpcService::builder()
        .endpoint("/rpc")
        .async_context_builder(move |request| {
            let context_builder = context_builder.clone();

            async move { context_builder.build(request).await }
        })
        .build()
}
