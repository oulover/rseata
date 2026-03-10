mod impl_branch_manager_inbound;
mod impl_branch_manager_outbound;
mod impl_resource_registry;
mod impl_branch_transaction_registry;

use async_trait::async_trait;
use rseata_core::types::{ClientId, GlobalStatus, ResourceId, Xid};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;

use rseata_core::branch::branch_transaction::BranchTransaction;
use rseata_core::branch::{BranchId, BranchType};
use rseata_core::grpc_client::rm_grpc_client::LazyRMGrpcClient;
use rseata_core::grpc_client::GrpcContext;
use rseata_core::handle_branch_type::HandleBranchType;
use rseata_core::resource::resource_manager::{GlobalStatusQuery, ResourceManager};
use rseata_core::resource::resource_registry::ResourceRegistry;
use rseata_core::resource::Resource;
use rseata_proto::rseata_proto::proto::{
    ResourceInstruction,
    ResourceProto,
};
use tokio::sync::mpsc::{Receiver, Sender};
use tonic::codegen::tokio_stream::StreamExt;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct ResourceInfo {
    resource_group_id: String,
    resource_id: ResourceId,
    branch_type: BranchType,
    client_id: ClientId,
}
impl ResourceInfo {
    pub fn new_with_env() -> Self {
        let resource_group_id = env::var("RSEATA_RM_RESOURCE_GROUP_ID")
            .unwrap_or("RSEATA_RM_RESOURCE_GROUP_ID".to_owned());
        let resource_id =
            env::var("RSEATA_RM_RESOURCE_ID").expect("env RSEATA_RM_RESOURCE_ID not set");
        // Parse branch type from environment variable, default to AT
        let branch_type = env::var("RSEATA_BRANCH_TYPE")
            .unwrap_or_else(|_| "AT".to_string())
            .to_uppercase();
        let branch_type = match branch_type.as_str() {
            "AT" => BranchType::AT,
            "XA" => BranchType::XA,
            "TCC" => BranchType::TCC,
            "SAGA" => BranchType::SAGA,
            _ => BranchType::AT, // default fallback
        };
        Self {
            resource_group_id,
            resource_id: ResourceId::from(resource_id),
            branch_type,
            client_id: ClientId::from(Uuid::new_v4().as_u128() as u64),
        }
    }

    pub fn branch_type(&self) -> BranchType {
        self.branch_type
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
    pub branch_transactions: Arc<RwLock<HashMap<BranchId, Box<dyn BranchTransaction + Send + Sync + 'static>>>>,
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
        self.resource_info.branch_type()
    }
}

#[async_trait]
impl GlobalStatusQuery for DefaultResourceManager {
    async fn get_global_status(&self, xid: Xid) -> anyhow::Result<GlobalStatus> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_resource_info_branch_type_from_env() {
        // Save original values
        let original_resource_id = env::var("RSEATA_RM_RESOURCE_ID").ok();
        let original_group_id = env::var("RSEATA_RM_RESOURCE_GROUP_ID").ok();
        let original_branch_type = env::var("RSEATA_BRANCH_TYPE").ok();

        unsafe {
            env::set_var("RSEATA_RM_RESOURCE_ID", "test-resource");
            env::set_var("RSEATA_RM_RESOURCE_GROUP_ID", "test-group");
        }

        // Test default (AT)
        unsafe {
            env::remove_var("RSEATA_BRANCH_TYPE");
        }
        let info = ResourceInfo::new_with_env();
        assert_eq!(info.branch_type(), BranchType::AT);

        // Test XA
        unsafe {
            env::set_var("RSEATA_BRANCH_TYPE", "XA");
        }
        let info = ResourceInfo::new_with_env();
        assert_eq!(info.branch_type(), BranchType::XA);

        // Test TCC
        unsafe {
            env::set_var("RSEATA_BRANCH_TYPE", "TCC");
        }
        let info = ResourceInfo::new_with_env();
        assert_eq!(info.branch_type(), BranchType::TCC);

        // Test SAGA
        unsafe {
            env::set_var("RSEATA_BRANCH_TYPE", "SAGA");
        }
        let info = ResourceInfo::new_with_env();
        assert_eq!(info.branch_type(), BranchType::SAGA);

        // Test unknown falls back to AT
        unsafe {
            env::set_var("RSEATA_BRANCH_TYPE", "UNKNOWN");
        }
        let info = ResourceInfo::new_with_env();
        assert_eq!(info.branch_type(), BranchType::AT);

        // Restore env
        if let Some(val) = original_resource_id {
            unsafe {
                env::set_var("RSEATA_RM_RESOURCE_ID", val);
            }
        } else {
            unsafe {
                env::remove_var("RSEATA_RM_RESOURCE_ID");
            }
        }
        if let Some(val) = original_group_id {
            unsafe {
                env::set_var("RSEATA_RM_RESOURCE_GROUP_ID", val);
            }
        } else {
            unsafe {
                env::remove_var("RSEATA_RM_RESOURCE_GROUP_ID");
            }
        }
        if let Some(val) = original_branch_type {
            unsafe {
                env::set_var("RSEATA_BRANCH_TYPE", val);
            }
        } else {
            unsafe {
                env::remove_var("RSEATA_BRANCH_TYPE");
            }
        }
    }
}
