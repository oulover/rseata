use crate::sea_orm::at::transaction_proxy::ATTransactionProxy;
use sea_orm::{DbErr, TransactionSession};

#[async_trait::async_trait]
impl TransactionSession for ATTransactionProxy {
    async fn commit(self) -> Result<(), DbErr> {
        self.branch_register().await?;
        self.prepare_undo_log().await?;
        let lucked = self.check_luck().await?;
        if !lucked {
            return Err(DbErr::Custom("luck error".to_string()));
        }
        let r = self.sea_transaction.commit().await;
        ATTransactionProxy::global_commit(r).await
    }

    async fn rollback(self) -> Result<(), DbErr> {
        // rollback 之前准备 undo log
        self.branch_register().await?;
        let r = self.sea_transaction.rollback().await;
        ATTransactionProxy::global_rollback().await?;
        r
    }
}
