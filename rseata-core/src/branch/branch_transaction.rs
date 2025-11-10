use crate::branch::branch_manager_inbound::BranchManagerInbound;
use crate::branch::{BranchId, BranchType};
use crate::types::{ClientId, ResourceId, Xid};
use async_trait::async_trait;

#[async_trait]
pub trait BranchTransaction: BranchManagerInbound + Send + Sync {}

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
