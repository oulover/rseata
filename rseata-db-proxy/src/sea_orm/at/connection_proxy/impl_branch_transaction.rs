use crate::sea_orm::at::connection_proxy::ATConnectionProxy;
use async_trait::async_trait;
use rseata_core::branch::branch_manager_inbound::BranchManagerInbound;
use rseata_core::branch::branch_transaction::BranchTransaction;
use rseata_core::branch::{BranchId, BranchStatus, BranchType};
use rseata_core::types::{ResourceId, Xid};

#[async_trait]
impl BranchManagerInbound for ATConnectionProxy {
    async fn branch_commit(
        &self,
        branch_type: BranchType,
        xid: Xid,
        branch_id: BranchId,
        resource_id: ResourceId,
        application_data: String,
    ) -> anyhow::Result<BranchStatus> {
        tracing::info!("PhaseTwoCommitted branch_commit");
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
        tracing::info!("PhaseTwoRollbacked branch_rollback");


        Ok(BranchStatus::PhaseTwoRollbacked)
    }
}

impl BranchTransaction for ATConnectionProxy {}
