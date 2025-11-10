mod impl_branch_manager_inbound;
mod impl_branch_manager_outbound;
mod impl_resource_registry;
mod impl_branch_transaction_registry;

use async_trait::async_trait;
use rseata_core::types::{ClientId, GlobalStatus, ResourceId, Xid};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};

use rseata_core::branch::branch_manager_inbound::BranchManagerInbound;
use rseata_core::branch::branch_manager_outbound::BranchManagerOutbound;
use rseata_core::branch::branch_transaction::{BranchTransaction, BranchTransactionRegistry};
use rseata_core::branch::{BranchId, BranchStatus, BranchType};
use rseata_core::grpc_client::GrpcContext;
use rseata_core::grpc_client::rm_grpc_client::LazyRMGrpcClient;
use rseata_core::handle_branch_type::HandleBranchType;
use rseata_core::resource::Resource;
use rseata_core::resource::resource_manager::{GlobalStatusQuery, ResourceManager};
use rseata_core::resource::resource_registry::ResourceRegistry;
use rseata_proto::rseata_proto::proto::resource_instruction::Instruction;
use rseata_proto::rseata_proto::proto::{
    BranchRegisterRequest, BranchReportRequest, LockQueryRequest, ResourceInstruction,
    ResourceProto,
};
use tokio::sync::mpsc::{Receiver, Sender};
use tonic::codegen::tokio_stream::StreamExt;
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct ResourceInfo {
    resource_group_id: String,
    resource_id: ResourceId,
    branch_type: BranchType,
    client_id: ClientId,
}
impl ResourceInfo {
    pub fn new_with_env(branch_type: BranchType) -> Self {
        let resource_group_id = env::var("RSEATA_RM_RESOURCE_GROUP_ID")
            .unwrap_or("RSEATA_RM_RESOURCE_GROUP_ID".to_owned());
        let resource_id =
            env::var("RSEATA_RM_RESOURCE_ID").expect("env RSEATA_RM_RESOURCE_ID not set");
        Self {
            resource_group_id,
            resource_id: ResourceId::from(resource_id),
            branch_type,
            client_id: ClientId::from(Uuid::new_v4().as_u128() as u64),
        }
    }
}
#[async_trait]
impl Resource for ResourceInfo {
    async fn get_resource_group_id(&self) -> String {
        self.resource_group_id.clone()
    }

    async fn get_resource_id(&self) -> ResourceId {
        self.resource_id.clone()
    }

    async fn get_branch_type(&self) -> BranchType {
        self.branch_type.clone()
    }

    async fn get_client_id(&self) -> ClientId {
        self.client_id
    }
}

fn get_tc_grpc_server_addr() -> String {
    let ip = env::var("RSEATA_TC_GRPC_IP").unwrap_or("127.0.0.1".to_string());
    let prot = env::var("RSEATA_TC_GRPC_PROT").unwrap_or("9811".to_string());
    format!("tcp://{}:{}", ip, prot)
}

#[derive(Clone)]
pub struct DefaultResourceManager {
    rm_client: LazyRMGrpcClient,
    resources: Arc<RwLock<HashMap<ResourceId, Box<ResourceInfo>>>>,
    channel: Arc<RwLock<Option<(Sender<ResourceProto>, Receiver<ResourceInstruction>)>>>,
    pub resource_info: ResourceInfo,
    pub branch_transactions :Arc<RwLock<HashMap<BranchId, Box<dyn BranchTransaction>>>>,
}
impl DefaultResourceManager {
    pub fn new(resource_info: ResourceInfo) -> Self {
        Self {
            rm_client: LazyRMGrpcClient::new(GrpcContext {
                endpoint: get_tc_grpc_server_addr(),
            }),
            resources: Arc::new(Default::default()),
            channel: Arc::new(RwLock::new(Default::default())),
            resource_info,
            branch_transactions: Arc::new(Default::default()),
        }
    }
    pub async fn init(&self) {
        self.register_resource(&self.resource_info).await;
    }
}


#[async_trait]
impl ResourceManager for DefaultResourceManager {
    async fn get_managed_resources(&self) -> Vec<Self::Resource> {
        todo!()
    }

    async fn find_resource(&self, resource_id: &ResourceId) -> Option<Self::Resource> {
        todo!()
    }
}

impl HandleBranchType for DefaultResourceManager {
    fn handle_branch_type(&self) -> BranchType {
        todo!()
    }
}

#[async_trait]
impl GlobalStatusQuery for DefaultResourceManager {
    async fn get_global_status(&self, xid: Xid) -> anyhow::Result<GlobalStatus> {
        todo!()
    }
}
