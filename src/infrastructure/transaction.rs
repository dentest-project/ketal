pub mod sqlx_transaction_manager;

#[cfg(test)]
mod transaction_test;

use crate::business::error::{GatewayError, UnexpectedError};
use sqlx_transaction_manager::SqlxTransactionConnection;
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, OnceLock},
};

static DEFAULT_TRANSACTION_MANAGER: OnceLock<Arc<dyn TransactionManager>> = OnceLock::new();

/// Called once during application setup. Tests select their own backend with a task scope.
pub(crate) fn configure_postgres_transactions() {
    DEFAULT_TRANSACTION_MANAGER
        .get_or_init(|| Arc::new(sqlx_transaction_manager::SqlxTransactionManager));
}

#[derive(Debug, thiserror::Error)]
pub enum TransactionError {
    #[error(transparent)]
    Gateway(#[from] GatewayError),
    #[error(transparent)]
    Unexpected(#[from] UnexpectedError),
}

impl From<sqlx::Error> for TransactionError {
    fn from(error: sqlx::Error) -> Self {
        Self::Gateway(error.into())
    }
}

impl TransactionError {
    fn into_use_case_error<E>(self) -> E
    where
        E: From<GatewayError> + From<UnexpectedError>,
    {
        match self {
            Self::Gateway(error) => error.into(),
            Self::Unexpected(error) => error.into(),
        }
    }
}

pub type TransactionFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, TransactionError>> + Send + 'a>>;

pub trait TransactionManager: Send + Sync {
    fn begin(&self) -> TransactionFuture<'_, Arc<dyn Transaction>>;
}

pub trait Transaction: Send + Sync {
    fn connection(&self) -> Result<SqlxTransactionConnection, UnexpectedError>;
    fn commit(self: Arc<Self>) -> TransactionFuture<'static, ()>;
    fn rollback(self: Arc<Self>) -> TransactionFuture<'static, ()>;
}

tokio::task_local! {
    static TRANSACTION_MANAGER: Arc<dyn TransactionManager>;
    static CURRENT_TRANSACTION: Arc<dyn Transaction>;
}

#[cfg(test)]
pub(crate) async fn with_transaction_manager<T>(
    manager: Arc<dyn TransactionManager>,
    operation: impl Future<Output = T>,
) -> T {
    TRANSACTION_MANAGER.scope(manager, operation).await
}

fn transaction_manager() -> Result<Arc<dyn TransactionManager>, UnexpectedError> {
    TRANSACTION_MANAGER.try_with(Arc::clone).or_else(|_| {
        DEFAULT_TRANSACTION_MANAGER
            .get()
            .cloned()
            .ok_or(UnexpectedError)
    })
}

pub(crate) fn current_transaction() -> Result<Arc<dyn Transaction>, UnexpectedError> {
    CURRENT_TRANSACTION
        .try_with(Arc::clone)
        .map_err(|_| UnexpectedError)
}

pub(crate) async fn run_in_transaction<T, E>(
    operation: impl Future<Output = Result<T, E>>,
) -> Result<T, E>
where
    E: From<GatewayError> + From<UnexpectedError>,
{
    // Unit tests inject plain gateways. Only infrastructure tests select a transaction backend.
    #[cfg(test)]
    if TRANSACTION_MANAGER.try_with(|_| ()).is_err() {
        return operation.await;
    }

    // Nested transactional use cases participate in their caller's transaction.
    if CURRENT_TRANSACTION.try_with(|_| ()).is_ok() {
        return operation.await;
    }

    let transaction = transaction_manager()?
        .begin()
        .await
        .map_err(TransactionError::into_use_case_error::<E>)?;
    let result = CURRENT_TRANSACTION
        .scope(Arc::clone(&transaction), operation)
        .await;

    match result {
        Ok(output) => {
            transaction
                .commit()
                .await
                .map_err(TransactionError::into_use_case_error::<E>)?;
            Ok(output)
        }
        Err(error) => {
            transaction
                .rollback()
                .await
                .map_err(TransactionError::into_use_case_error::<E>)?;
            Err(error)
        }
    }
}
