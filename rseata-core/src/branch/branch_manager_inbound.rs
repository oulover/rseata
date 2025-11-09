use crate::branch::{BranchId, BranchStatus, BranchType};
use crate::types::{ResourceId, Xid};
use tonic::async_trait;

#[async_trait]
pub trait BranchManagerInbound {
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
