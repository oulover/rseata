use crate::sea_orm::xa::transaction_proxy::{TransactionType, XATransactionProxy};
use sea_orm::{DbErr, ExecResult, TransactionSession};

#[async_trait::async_trait]
impl TransactionSession for XATransactionProxy {
    async fn commit(self) -> Result<(), DbErr> {
        match self.transaction_type {
            TransactionType::Local(local) => local.commit().await,
            TransactionType::XA(ref xa_id) => {
                self.branch_register().await?;
                let end_result = self.xa_end(xa_id).await;
                let lucked = self.check_lock().await?;
                if !lucked {
                    tracing::error!("Check lock failed");
                    return self.rollback().await;
                }

                match end_result {
                    Ok(_) => {
                        let prepare_result = self.xa_prepare(xa_id).await;
                        match prepare_result {
                            Ok(_) => XATransactionProxy::global_commit(prepare_result)
                                .await
                                .map(|_| ()),
                            Err(_) => self.rollback().await,
                        }
                    }
                    Err(_) => self.rollback().await,
                }
                .map(|_| ())
            }
        }
    }

    async fn rollback(self) -> Result<(), DbErr> {
        match self.transaction_type {
            TransactionType::Local(local) => local.rollback().await,
            TransactionType::XA(ref xa_id) => {
                self.branch_register().await?;
                let end_result = self.xa_rollback(xa_id).await;
                let _ = XATransactionProxy::global_rollback().await?;
                end_result.map(|_| ())
            }
        }
    }
}
