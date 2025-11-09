use crate::branch::branch_manager_inbound::BranchManagerInbound;
use crate::branch::branch_manager_outbound::BranchManagerOutbound;
use crate::handle_branch_type::HandleBranchType;
use crate::resource::resource_registry::ResourceRegistry;
use crate::types::{GlobalStatus, ResourceId, Xid};
use tonic::async_trait;

#[async_trait]
pub trait GlobalStatusQuery {
    async fn get_global_status(&self, xid: Xid) -> anyhow::Result<GlobalStatus>;
}

#[async_trait]
pub trait ResourceManager:
    ResourceRegistry
    + HandleBranchType
    + GlobalStatusQuery
    + BranchManagerInbound
    + BranchManagerOutbound
{
    async fn get_managed_resources(&self) -> Vec<Self::Resource>;
    async fn find_resource(&self, resource_id: &ResourceId) -> Option<Self::Resource>;
}
