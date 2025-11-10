use crate::sea_orm::xa::connection_proxy::XAConnectionProxy;
use crate::sea_orm::xa::transaction_proxy::{TransactionType, XATransactionProxy};
use async_trait::async_trait;
use rseata_core::branch::branch_manager_inbound::BranchManagerInbound;
use rseata_core::branch::branch_transaction::BranchTransaction;
use rseata_core::branch::{BranchId, BranchStatus, BranchType};
use rseata_core::types::{ResourceId, Xid};
use sea_orm::{DbErr, ExecResult, TransactionSession};

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
        self.branch_commit(branch_type, xid, branch_id, resource_id, application_data)
            .await?;
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
        self.branch_rollback(branch_type, xid, branch_id, resource_id, application_data)
            .await?;
        Ok(BranchStatus::PhaseTwoCommitted)
    }
}
