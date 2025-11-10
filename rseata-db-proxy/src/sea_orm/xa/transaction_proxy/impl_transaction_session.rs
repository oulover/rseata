use crate::sea_orm::xa::transaction_proxy::{TransactionType, XATransactionProxy};
use sea_orm::{DbErr, ExecResult, TransactionSession};

#[async_trait::async_trait]
impl TransactionSession for XATransactionProxy {
    async fn commit(self) -> Result<(), DbErr> {
        match self.transaction_type {
            TransactionType::Local(local) => {
                let r = local.lock().await.take();
                if let Some( r) = r {
                    r.commit().await
                }else { 
                    Ok(())
                }
            },
            TransactionType::XA(ref xa_id, ref conn) => {
                tracing::info!("-----------------------------1--");
                self.branch_register(&xa_id).await?;
                tracing::info!("-----------------------------2--");
                let end_result = self.xa_end(conn.clone(),xa_id).await;
                tracing::info!("-------------------------end_result----3--{:?}",end_result);
                let lucked = self.check_lock().await?;
                if !lucked {
                    tracing::error!("Check lock failed");
                    return self.rollback().await;
                }

                match end_result {
                    Ok(_) => {
                        let prepare_result = self.xa_prepare(conn.clone(),xa_id).await;
                        tracing::info!("-------------------------prepare_result----4--{:?}",prepare_result);
                        
                        match prepare_result {
                            Ok(_) => XATransactionProxy::global_commit(prepare_result)
                                .await,
                            Err(_) => self.rollback().await,
                        }
                    }
                    Err(_) => self.rollback().await,
                }
            }
        }
    }

    async fn rollback(self) -> Result<(), DbErr> {

        match self.transaction_type {
            TransactionType::Local(local) => {
                let r = local.lock().await.take();
                if let Some( r) = r {
                    r.rollback().await
                }else {
                    Ok(())
                }
            },
            TransactionType::XA(ref xa_id,  ref conn) => {
                self.branch_register(xa_id).await?;
                let end_result = self.xa_rollback_xa_id(conn.clone(),xa_id).await;
                let _ = XATransactionProxy::global_rollback().await?;
                end_result.map(|_| ())
            }
        }
    }
}
