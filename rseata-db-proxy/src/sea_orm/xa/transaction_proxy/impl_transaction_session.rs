use crate::sea_orm::xa::transaction_proxy::{TransactionType, XATransactionProxy};
use sea_orm::{DbErr, ExecResult, TransactionSession};

#[async_trait::async_trait]
impl TransactionSession for XATransactionProxy {
    async fn commit(self) -> Result<(), DbErr> {
        match self.transaction_type {
            TransactionType::Local(local) => {
                let r = local.lock().await.take();
                if let Some(r) = r {
                    r.commit().await
                } else {
                    Ok(())
                }
            }
            TransactionType::XA(ref xa_transaction) => {
                self.branch_register().await?;
                let end_result = xa_transaction.xa_end().await;
                let lucked = self.check_lock().await?;
                if !lucked {
                    tracing::error!("Check lock failed");
                    return self.rollback().await;
                }

                match end_result {
                    Ok(_) => {
                        let prepare_result = xa_transaction.xa_prepare().await;
                        tracing::info!(
                            "-------------------------prepare_result----4--{:?}",
                            prepare_result
                        );

                        match prepare_result {
                            Ok(_) => XATransactionProxy::report_local_commit(prepare_result).await,
                            Err(_) => self.rollback().await,
                        }
                    }
                    Err(_) => self.rollback().await,
                }
            }
        }
    }

    async fn rollback(self) -> Result<(), DbErr> {
        match &self.transaction_type {
            TransactionType::Local(local) => {
                let r = local.lock().await.take();
                if let Some(r) = r {
                    r.rollback().await
                } else {
                    Ok(())
                }
            }
            TransactionType::XA(xa_transaction) => {
                self.branch_register().await?;
                let end_result = xa_transaction.xa_rollback().await;
                let _ = XATransactionProxy::global_rollback().await?;
                end_result.map(|_| ())
            }
        }
    }
}
