use crate::sea_orm::xa::transaction_proxy::{TransactionType, XATransactionProxy};
use async_trait::async_trait;
use rseata_core::branch::branch_transaction::BranchTransaction;
use rseata_core::branch::{BranchId, BranchStatus, BranchType};
use rseata_core::types::{ResourceId, Xid};

#[async_trait]
impl BranchTransaction for XATransactionProxy {
    async fn branch_commit(
        &self,
        branch_type: BranchType,
        xid: Xid,
        branch_id: BranchId,
        resource_id: ResourceId,
        application_data: String,
    ) -> anyhow::Result<BranchStatus> {
        tracing::info!("XA branch_commit ing :{xid},{branch_id}",);

        match &self.transaction_type {
            TransactionType::Local(local) => {
                let t = local.lock().await.take();
                if let Some(t) = t {
                    t.commit().await?;
                }
            }
            TransactionType::XA(xa_transaction) => {
                xa_transaction.xa_commit().await?;
            }
        }
        Ok(BranchStatus::PhaseTwoCommitted)
    }

    async fn branch_rollback(
        &self,
        branch_type: BranchType,
        xid: Xid,
        branch_id: BranchId,
        resource_id: ResourceId,
        application_data: String,
    ) -> anyhow::Result<BranchStatus> {
        match &self.transaction_type {
            TransactionType::Local(local) => {
                let t = local.lock().await.take();
                if let Some(t) = t {
                    t.rollback().await?;
                }
            }
            TransactionType::XA(xa_transaction) => {
                xa_transaction.xa_rollback().await?;
            }
        }
        Ok(BranchStatus::PhaseTwoCommitted)
    }
}
