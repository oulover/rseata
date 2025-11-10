use crate::branch::BranchType;
use crate::types::{ClientId, ResourceId};
use tonic::async_trait;

pub mod resource_manager;
pub mod resource_registry;

#[async_trait]
pub trait Resource {
    async fn get_resource_group_id(&self) -> String;
    async fn get_resource_id(&self) -> ResourceId;
    async fn get_branch_type(&self) -> BranchType;
    async fn get_client_id(&self) -> ClientId;
}
#[derive(Clone, Debug)]
pub struct DefaultResource {
    pub group_id: String,
    pub resource_id: ResourceId,
    pub branch_type: BranchType,
    pub client_id: ClientId,
}
impl DefaultResource {
    pub fn new(
        group_id: String,
        resource_id: ResourceId,
        branch_type: BranchType,
        client_id: ClientId,
    ) -> Self {
        Self {
            group_id,
            resource_id,
            branch_type,
            client_id,
        }
    }
}
#[async_trait]
impl Resource for DefaultResource {
    async fn get_resource_group_id(&self) -> String {
        self.group_id.clone()
    }

    async fn get_resource_id(&self) -> ResourceId {
        self.resource_id.clone()
    }

    async fn get_branch_type(&self) -> BranchType {
        self.branch_type
    }

    async fn get_client_id(&self) -> ClientId {
        self.client_id
    }
}
