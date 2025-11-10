use async_trait::async_trait;
use rseata_core::types::{ClientId, GlobalStatus, ResourceId, Xid};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};

use rseata_core::branch::branch_manager_inbound::BranchManagerInbound;
use rseata_core::branch::branch_manager_outbound::BranchManagerOutbound;
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
    pub fn new(resource_group_id: String, branch_type: BranchType) -> Self {
        Self {
            resource_group_id,
            resource_id: ResourceId::from("ResourceId5555"),
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

#[async_trait]
impl ResourceRegistry for DefaultResourceManager {
    type Resource = ResourceInfo;

    async fn register_resource(&self, resource: &Self::Resource) {
        let resource_clone = resource.clone();
        // 注册流到远程
        {
            let (request_tx, request_rx) = mpsc::channel(100);
            let (response_tx, response_rx) = mpsc::channel(100);

            let request_stream = ReceiverStream::new(request_rx);

            // 启动持久的流调用
            let response_stream = self
                .rm_client
                .get()
                .await
                .unwrap()
                .rm
                .register_resource(tonic::Request::new(request_stream))
                .await
                .unwrap()
                .into_inner();

            let self_cloned = self.clone();
            // 处理响应流的后台任务
            tokio::spawn(async move {
                let mut response_stream = response_stream;
                while let Some(response) = response_stream.next().await {
                    match response {
                        Ok(branch_response) => {
                            if let Some(instruction) = branch_response.instruction {
                                println!("------------Received instruction: {:?}", instruction);
                                match instruction {
                                    Instruction::Commit(commit) => {
                                        self_cloned
                                            .branch_commit(
                                                commit.branch_type.into(),
                                                commit.xid.into(),
                                                commit.branch_id.into(),
                                                commit.resource_id.into(),
                                                commit.application_data,
                                            )
                                            .await
                                            .ok();
                                    }
                                    Instruction::Rollback(rollback) => {
                                        self_cloned
                                            .branch_rollback(
                                                rollback.branch_type.into(),
                                                rollback.xid.into(),
                                                rollback.branch_id.into(),
                                                rollback.resource_id.into(),
                                                rollback.application_data,
                                            )
                                            .await
                                            .ok();
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Error in branch register stream: {}", e);
                            break;
                        }
                    }
                }
            });

            // Resource Registry
            {
                let _ = request_tx
                    .send(ResourceProto {
                        resource_group_id: resource_clone.resource_group_id,
                        resource_id: resource_clone.resource_id.0,
                        client_id: resource_clone.client_id.into(),
                        branch_type: resource_clone.branch_type.into(),
                    })
                    .await;
            }

            // 添加本地
            let mut channel = self.channel.write().await;
            *channel = Some((request_tx, response_rx));
        }

        // 添加本地
        let mut resources = self.resources.write().await;
        resources.insert(resource.get_resource_id().await, Box::new(resource.clone()));
    }

    async fn unregister_resource(&mut self, resource: &Self::Resource) {}
}
#[async_trait]
impl BranchManagerOutbound for DefaultResourceManager {
    async fn branch_register(
        &self,
        branch_type: BranchType,
        resource_id: ResourceId,
        client_id: ClientId,
        xid: Xid,
        application_data: String,
        lock_keys: String,
    ) -> anyhow::Result<BranchId> {
        let request = BranchRegisterRequest {
            branch_type: branch_type.into(),
            resource_id: resource_id.0,
            client_id: client_id.into(),
            xid: xid.to_string(),
            application_data,
            lock_keys,
        };
        let response = self
            .rm_client
            .get()
            .await?
            .rm
            .branch_register(request)
            .await?;
        Ok(response.into_inner().branch_id.into())
    }

    async fn branch_report(
        &self,
        branch_type: BranchType,
        xid: Xid,
        branch_id: BranchId,
        status: BranchStatus,
        application_data: String,
    ) -> anyhow::Result<()> {
        self.rm_client
            .get()
            .await?
            .rm
            .branch_report(BranchReportRequest {
                branch_type: branch_type.into(),
                xid: xid.to_string(),
                branch_id: branch_id.into(),
                status: status.into(),
                application_data,
            })
            .await?;

        Ok(())
    }

    async fn lock_query(
        &self,
        branch_type: BranchType,
        resource_id: ResourceId,
        xid: Xid,
        lock_keys: String,
    ) -> anyhow::Result<bool> {
        let r = self
            .rm_client
            .get()
            .await?
            .rm
            .lock_query(LockQueryRequest {
                branch_type: branch_type.into(),
                resource_id: resource_id.0,
                xid: xid.to_string(),
                lock_keys,
            })
            .await?;
        Ok(r.into_inner().locked)
    }
}
#[async_trait]
impl BranchManagerInbound for DefaultResourceManager {
    async fn branch_commit(
        &self,
        branch_type: BranchType,
        xid: Xid,
        branch_id: BranchId,
        resource_id: ResourceId,
        application_data: String,
    ) -> anyhow::Result<BranchStatus> {
        // todo: branch commit local transaction
        println!("BranchManagerInbound branch_commit------");
        let _ = self
            .branch_report(
                branch_type,
                xid,
                branch_id,
                BranchStatus::PhaseTwoCommitted,
                application_data,
            )
            .await;
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
        // todo: branch rollback local transaction
        println!("BranchManagerInbound branch_rollback------");
        let _ = self
            .branch_report(
                branch_type,
                xid,
                branch_id,
                BranchStatus::PhaseTwoRollbacked,
                application_data,
            )
            .await;
        Ok(BranchStatus::PhaseTwoRollbacked)
    }
}
