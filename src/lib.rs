mod database;
mod infrastructure;

pub mod business;

use jsonrpc_usecase::{JsonRpcService, RegistrationError};

pub fn build_service() -> Result<JsonRpcService, RegistrationError> {
    JsonRpcService::builder().endpoint("/rpc").build()
}
