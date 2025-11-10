use async_trait::async_trait;
use lazy_static::lazy_static;
use rseata_core::grpc_client::GrpcContext;
use rseata_core::grpc_client::tm_grpc_client::LazyTMGrpcClient;
use rseata_core::transaction::transaction_manager::TransactionManager;
use rseata_core::types::{GlobalStatus, Xid};
use rseata_proto::rseata_proto::proto::{
    GlobalBeginRequest, GlobalCommitRequest, GlobalReportRequest, GlobalRollbackRequest,
};
use std::env;
use std::sync::Arc;

pub fn get_tc_grpc_server_addr() -> String {
    let ip = env::var("RSEATA_TC_GRPC_IP").unwrap_or("127.0.0.1".to_string());
    let prot = env::var("RSEATA_TC_GRPC_PROT").unwrap_or("9811".to_string());
    format!("tcp://{}:{}", ip, prot)
}

lazy_static! {
    static ref TM_GRPC_CLIENT: LazyTMGrpcClient = LazyTMGrpcClient::new(GrpcContext {
        endpoint: get_tc_grpc_server_addr()
    });
}
#[derive(Clone, Debug)]
pub struct RseataTM {
    pub application_id: Arc<String>,
    pub transaction_service_group: Arc<String>,
}
impl RseataTM {
    pub fn new_with_env() -> Self {
        let app =
            env::var("RSEATA_TM_APPLICATION_ID").expect("env RSEATA_TM_APPLICATION_ID not set");
        let group = env::var("RSEATA_TM_TRANSACTION_SERVICE_GROUP")
            .unwrap_or("RSEATA_TM_TRANSACTION_SERVICE_GROUP".to_owned());
        Self {
            application_id: Arc::new(app),
            transaction_service_group: Arc::new(group),
        }
    }
}

#[async_trait]
impl TransactionManager for RseataTM {
    async fn begin(
        &self,
        application_id: String,
        transaction_service_group: String,
        name: String,
        timeout_millis: u64,
    ) -> anyhow::Result<Xid> {
        let r = TM_GRPC_CLIENT
            .get()
            .await?
            .tc
            .global_begin(GlobalBeginRequest {
                application_id,
                transaction_service_group,
                transaction_name: name,
                timeout_millis,
                extra_data: None,
            })
            .await?;
        let xid = r.into_inner().xid.clone();
        Ok(Xid::from(xid))
    }

    async fn commit(&self, xid: Xid) -> anyhow::Result<GlobalStatus> {
        tracing::info!("TM commit {}", xid);
        TM_GRPC_CLIENT
            .get()
            .await?
            .tc
            .global_commit(GlobalCommitRequest {
                xid: xid.to_string(),
                extra_data: None,
            })
            .await?;
        Ok(GlobalStatus::Committed)
    }

    async fn rollback(&self, xid: Xid) -> anyhow::Result<GlobalStatus> {
        println!("global_rollback--{}", xid);
        TM_GRPC_CLIENT
            .get()
            .await?
            .tc
            .global_rollback(GlobalRollbackRequest {
                xid: xid.to_string(),
                extra_data: None,
            })
            .await?;
        Ok(GlobalStatus::Rollbacked)
    }

    async fn get_status(&self, xid: Xid) -> anyhow::Result<GlobalStatus> {
        println!("get_status--{}", xid);
        Ok(GlobalStatus::Finished)
    }

    async fn global_report(
        &self,
        xid: &Xid,
        global_status: GlobalStatus,
    ) -> anyhow::Result<GlobalStatus> {
        let status = TM_GRPC_CLIENT
            .get()
            .await?
            .tc
            .global_report(GlobalReportRequest {
                xid: xid.to_string(),
                global_status: global_status.code(),
            })
            .await?;
        Ok(GlobalStatus::from_code(status.into_inner().global_status)
            .map_err(|e| anyhow::anyhow!(e))?)
    }
}
