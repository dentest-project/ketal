use super::{
    Transaction, TransactionError, TransactionFuture, TransactionManager, current_transaction,
    run_in_transaction, sqlx_transaction_manager::SqlxTransactionConnection,
    with_transaction_manager,
};
use crate::business::error::{GatewayError, UnexpectedError};
use macros::Transactional;
use std::{
    error::Error,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::sync::{Barrier, Notify};

struct Operation;

#[Transactional]
impl Operation {
    async fn execute(&self, result: Result<(), GatewayError>) -> Result<(), TransactionError> {
        result?;
        Ok(())
    }
}

#[tokio::test]
async fn macro_commits_after_success() -> Result<(), Box<dyn Error>> {
    let manager = Arc::new(TransactionManagerDouble::default());

    with_transaction_manager(manager.clone(), Operation.execute(Ok(()))).await?;

    assert_eq!(manager.events()?, ["begin", "commit"]);
    assert!(current_transaction().is_err());
    Ok(())
}

#[tokio::test]
async fn macro_rolls_back_on_an_error() -> Result<(), Box<dyn Error>> {
    let manager = Arc::new(TransactionManagerDouble::default());

    let result = with_transaction_manager(
        manager.clone(),
        Operation.execute(Err(GatewayError::new("operation failed"))),
    )
    .await;

    assert!(matches!(result, Err(TransactionError::Gateway(_))));
    assert_eq!(manager.events()?, ["begin", "rollback"]);
    assert!(current_transaction().is_err());
    Ok(())
}

struct NestedOperation;

#[Transactional]
impl NestedOperation {
    async fn execute(&self) -> Result<(), TransactionError> {
        Operation.execute(Ok(())).await?;
        Err(GatewayError::new("outer operation failed").into())
    }
}

#[tokio::test]
async fn nested_operations_share_the_outer_transaction() -> Result<(), Box<dyn Error>> {
    let manager = Arc::new(TransactionManagerDouble::default());

    let result = with_transaction_manager(manager.clone(), NestedOperation.execute()).await;

    assert!(matches!(result, Err(TransactionError::Gateway(_))));
    assert_eq!(manager.events()?, ["begin", "rollback"]);
    Ok(())
}

#[tokio::test]
async fn transactions_are_isolated_between_concurrent_executions() -> Result<(), Box<dyn Error>> {
    let first = Arc::new(TransactionManagerDouble::default());
    let second = Arc::new(TransactionManagerDouble::default());
    let barrier = Barrier::new(2);

    let (first_result, second_result) = tokio::join!(
        with_transaction_manager(
            first.clone(),
            run_in_transaction(async {
                let transaction = current_transaction()?;
                barrier.wait().await;
                assert!(Arc::ptr_eq(&transaction, &current_transaction()?));
                Ok::<_, TransactionError>(())
            })
        ),
        with_transaction_manager(
            second.clone(),
            run_in_transaction(async {
                let transaction = current_transaction()?;
                barrier.wait().await;
                assert!(Arc::ptr_eq(&transaction, &current_transaction()?));
                Ok::<_, TransactionError>(())
            })
        ),
    );
    first_result?;
    second_result?;

    assert_eq!(first.events()?, ["begin", "commit"]);
    assert_eq!(second.events()?, ["begin", "commit"]);
    assert!(current_transaction().is_err());
    assert!(super::TRANSACTION_MANAGER.try_with(|_| ()).is_err());
    Ok(())
}

#[tokio::test]
async fn cancellation_drops_the_active_transaction() -> Result<(), Box<dyn Error>> {
    let manager = Arc::new(TransactionManagerDouble::default());
    let task_manager = manager.clone();
    let started = Arc::new(Notify::new());
    let task_started = started.clone();

    let task = tokio::spawn(async move {
        with_transaction_manager(
            task_manager,
            run_in_transaction(async {
                task_started.notify_one();
                std::future::pending::<()>().await;
                Ok::<_, TransactionError>(())
            }),
        )
        .await
    });
    started.notified().await;

    assert!(current_transaction().is_err());
    task.abort();
    assert!(matches!(task.await, Err(error) if error.is_cancelled()));
    assert_eq!(manager.events()?, ["begin"]);
    assert!(manager.dropped.load(Ordering::Relaxed));
    Ok(())
}

#[tokio::test]
async fn transaction_failures_are_propagated() -> Result<(), Box<dyn Error>> {
    for phase in ["begin", "commit", "rollback"] {
        let manager = Arc::new(TransactionManagerDouble {
            fail_at: Some(phase),
            ..Default::default()
        });
        let operation_result = if phase == "rollback" {
            Err(GatewayError::new("operation failed"))
        } else {
            Ok(())
        };

        let result =
            with_transaction_manager(manager.clone(), Operation.execute(operation_result)).await;

        assert!(matches!(result, Err(TransactionError::Gateway(_))));
        if let Err(error) = result {
            assert_eq!(error.to_string(), format!("{phase} failed"));
        }
        let expected = if phase == "begin" {
            vec!["begin"]
        } else {
            vec!["begin", phase]
        };
        assert_eq!(manager.events()?, expected);
        assert!(current_transaction().is_err());
    }
    Ok(())
}

#[derive(Default)]
struct TransactionManagerDouble {
    events: Arc<Mutex<Vec<&'static str>>>,
    dropped: Arc<AtomicBool>,
    fail_at: Option<&'static str>,
}

impl TransactionManagerDouble {
    fn events(&self) -> Result<Vec<&'static str>, UnexpectedError> {
        Ok(self.events.lock().map_err(|_| UnexpectedError)?.clone())
    }
}

impl TransactionManager for TransactionManagerDouble {
    fn begin(&self) -> TransactionFuture<'_, Arc<dyn Transaction>> {
        Box::pin(async move {
            self.events
                .lock()
                .map_err(|_| UnexpectedError)?
                .push("begin");
            if self.fail_at == Some("begin") {
                return Err(GatewayError::new("begin failed").into());
            }
            Ok(Arc::new(TransactionDouble {
                events: self.events.clone(),
                dropped: self.dropped.clone(),
                fail_at: self.fail_at,
            }) as Arc<dyn Transaction>)
        })
    }
}

struct TransactionDouble {
    events: Arc<Mutex<Vec<&'static str>>>,
    dropped: Arc<AtomicBool>,
    fail_at: Option<&'static str>,
}

impl Transaction for TransactionDouble {
    fn connection(&self) -> Result<SqlxTransactionConnection, UnexpectedError> {
        Err(UnexpectedError)
    }

    fn commit(self: Arc<Self>) -> TransactionFuture<'static, ()> {
        Box::pin(async move {
            self.events
                .lock()
                .map_err(|_| UnexpectedError)?
                .push("commit");
            if self.fail_at == Some("commit") {
                return Err(GatewayError::new("commit failed").into());
            }
            Ok(())
        })
    }

    fn rollback(self: Arc<Self>) -> TransactionFuture<'static, ()> {
        Box::pin(async move {
            self.events
                .lock()
                .map_err(|_| UnexpectedError)?
                .push("rollback");
            if self.fail_at == Some("rollback") {
                return Err(GatewayError::new("rollback failed").into());
            }
            Ok(())
        })
    }
}

impl Drop for TransactionDouble {
    fn drop(&mut self) {
        self.dropped.store(true, Ordering::Relaxed);
    }
}
