use crate::branch::branch_manager_inbound::BranchManagerInbound;
use crate::branch::{BranchId, BranchStatus, BranchType};
use crate::types::{ClientId, ResourceId, Xid};
use async_trait::async_trait;

#[async_trait]
pub trait BranchTransaction: Send + Sync + 'static {
    async fn branch_commit(
        &self,
        branch_type: BranchType,
        xid: Xid,
        branch_id: BranchId,
        resource_id: ResourceId,
        application_data: String,
    ) -> anyhow::Result<BranchStatus>;

    async fn branch_rollback(
        &self,
        branch_type: BranchType,
        xid: Xid,
        branch_id: BranchId,
        resource_id: ResourceId,
        application_data: String,
    ) -> anyhow::Result<BranchStatus>;
}

#[async_trait]
pub trait BranchTransactionRegistry {
    async fn branch_transaction_registry(
        &self,
        branch_type: BranchType,
        resource_id: ResourceId,
        client_id: ClientId,
        xid: Xid,
        application_data: String,
        lock_keys: String,
        branch_transaction: Box<dyn BranchTransaction>,
    ) -> anyhow::Result<BranchId>;
}
