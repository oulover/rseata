use crate::branch::{BranchId, BranchStatus, BranchType};
use crate::types::{ClientId, ResourceId, Xid};
use tonic::async_trait;

#[async_trait]
pub trait BranchManagerOutbound {
    async fn branch_register(
        &self,
        branch_type: BranchType,
        resource_id: ResourceId,
        client_id: ClientId,
        xid: Xid,
        application_data: String,
        lock_keys: String,
    ) -> anyhow::Result<BranchId>;

    async fn branch_report(
        &self,
        branch_type: BranchType,
        xid: Xid,
        branch_id: BranchId,
        status: BranchStatus,
        application_data: String,
    ) -> anyhow::Result<()>;

    async fn lock_query(
        &self,
        branch_type: BranchType,
        resource_id: ResourceId,
        xid: Xid,
        lock_keys: String,
    ) -> anyhow::Result<bool>;
}
