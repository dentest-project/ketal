use super::{
    Transaction, TransactionError, TransactionFuture, TransactionManager, current_transaction,
};
use crate::{business::error::UnexpectedError, database::shared_pg_pool};
use sqlx::Postgres;
use std::sync::Arc;
use tokio::sync::Mutex;

pub(crate) type SqlxTransactionConnection = Arc<Mutex<sqlx::Transaction<'static, Postgres>>>;

pub(crate) fn current_connection() -> Result<SqlxTransactionConnection, UnexpectedError> {
    current_transaction()?.connection()
}

#[derive(Default)]
pub struct SqlxTransactionManager;

impl TransactionManager for SqlxTransactionManager {
    fn begin(&self) -> TransactionFuture<'_, Arc<dyn Transaction>> {
        Box::pin(async {
            let connection = Arc::new(Mutex::new(shared_pg_pool().begin().await?));
            Ok(Arc::new(SqlxTransaction { connection }) as Arc<dyn Transaction>)
        })
    }
}

struct SqlxTransaction {
    connection: SqlxTransactionConnection,
}

impl SqlxTransaction {
    fn into_connection(
        self: Arc<Self>,
    ) -> Result<sqlx::Transaction<'static, Postgres>, TransactionError> {
        let transaction = Arc::try_unwrap(self).map_err(|_| UnexpectedError)?;
        Ok(Arc::try_unwrap(transaction.connection)
            .map_err(|_| UnexpectedError)?
            .into_inner())
    }
}

impl Transaction for SqlxTransaction {
    fn connection(&self) -> Result<SqlxTransactionConnection, UnexpectedError> {
        Ok(Arc::clone(&self.connection))
    }

    fn commit(self: Arc<Self>) -> TransactionFuture<'static, ()> {
        Box::pin(async move {
            self.into_connection()?.commit().await?;
            Ok(())
        })
    }

    fn rollback(self: Arc<Self>) -> TransactionFuture<'static, ()> {
        Box::pin(async move {
            self.into_connection()?.rollback().await?;
            Ok(())
        })
    }
}
