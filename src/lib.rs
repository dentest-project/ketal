mod database;

pub mod business;
pub mod infrastructure;

use jsonrpc_usecase::{JsonRpcService, RegistrationError};

pub fn build_service() -> Result<JsonRpcService, RegistrationError> {
    JsonRpcService::builder().endpoint("/rpc").build()
}
