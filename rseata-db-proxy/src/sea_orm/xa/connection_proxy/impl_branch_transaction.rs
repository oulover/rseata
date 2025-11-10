use crate::sea_orm::xa::connection_proxy::XAConnectionProxy;
use async_trait::async_trait;
use rseata_core::branch::branch_manager_inbound::BranchManagerInbound;
use rseata_core::branch::branch_transaction::BranchTransaction;
use rseata_core::branch::{BranchId, BranchStatus, BranchType};
use rseata_core::types::{ResourceId, Xid};
use sea_orm::{DbErr, ExecResult};

#[async_trait]
impl BranchManagerInbound for XAConnectionProxy {
    async fn branch_commit(
        &self,
        branch_type: BranchType,
        xid: Xid,
        branch_id: BranchId,
        resource_id: ResourceId,
        application_data: String,
    ) -> anyhow::Result<BranchStatus> {
        tracing::info!("XA branch_commit ing :{xid},{branch_id}",);


        let xa_r = self.xa_commit(&xid).await;
        return match xa_r {
            Ok(_) => {
                tracing::info!("XA branch_commit success :{xid},{branch_id}",);
                Ok(BranchStatus::PhaseTwoCommitted)
            }
            Err(e) => {
                tracing::error!("Failed to commit xa:{}", e.to_string());
                Ok(BranchStatus::PhaseTwoCommitFailedUnretryable)
            }
        };
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
        tracing::info!("XA branch_rollback ing :{xid},{branch_id}",);
        if let Some((xa_id,_)) = {self.xa_id.read().await}.as_ref() {
            let xa_r = self.xa_rollback(&xid).await;
            return match xa_r {
                Ok(_) => {
                    tracing::info!("XA branch_rollback success :{xid},{branch_id}",);
                    Ok(BranchStatus::PhaseTwoRollbacked)
                }
                Err(e) => Ok(BranchStatus::PhaseTwoRollbackFailedUnretryable),
            };
        }
        Ok(BranchStatus::PhaseTwoCommitted)
    }
}

impl BranchTransaction for XAConnectionProxy {}
