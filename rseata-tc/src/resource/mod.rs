use crate::types::ConnectionId;
use async_trait::async_trait;
use rseata_core::branch::BranchType;
use rseata_core::resource::{DefaultResource, Resource};
use rseata_core::types::{ClientId, ResourceId};
use rseata_proto::rseata_proto::proto::ResourceInstruction;
use tokio::sync::mpsc::Sender;
use tonic::Status;

#[derive(Clone)]
pub struct TCResource {
    pub connection_id: ConnectionId,
    pub resource: DefaultResource,
    pub response_tx: Sender<Result<ResourceInstruction, Status>>,
}

#[async_trait]
impl Resource for TCResource {
    async fn get_resource_group_id(&self) -> String {
        self.resource.get_resource_group_id().await
    }

    async fn get_resource_id(&self) -> ResourceId {
        self.resource.get_resource_id().await
    }

    async fn get_branch_type(&self) -> BranchType {
        self.resource.get_branch_type().await
    }

    async fn get_client_id(&self) -> ClientId {
        self.resource.get_client_id().await
    }
}
