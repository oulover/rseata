use async_trait::async_trait;
use rseata_core::transaction::transaction_manager::TransactionManager;
use rseata_core::types::{GlobalStatus, Xid};
use rseata_proto::rseata_proto::proto::{
    GlobalBeginRequest, GlobalCommitRequest, GlobalRollbackRequest,
};

pub static RSEATA_TM: RseataTM = RseataTM;
pub use futures::FutureExt;
use lazy_static::lazy_static;
pub use rseata_core::RSEATA_CLIENT_SESSION;
use rseata_core::grpc_client::grpc_client_impl::LazyTmGrpcClient;

pub struct RseataTM;

lazy_static! {
    static ref client: LazyTmGrpcClient = LazyTmGrpcClient::new(());
}
#[async_trait]
impl TransactionManager for RseataTM {
    async fn begin(
        &self,
        application_id: String,
        transaction_service_group: String,
        name: String,
        timeout: u64,
    ) -> anyhow::Result<Xid> {
        let r = client
            .get()
            .await?
            .tc
            .global_begin(GlobalBeginRequest {
                application_id: "".to_string(),
                transaction_service_group: "".to_string(),
                transaction_name: name,
                timeout_millis: timeout,
                extra_data: None,
            })
            .await?;
        let xid = r.into_inner().xid.clone();
        Ok(Xid::from(xid))
    }

    async fn commit(&self, xid: Xid) -> anyhow::Result<GlobalStatus> {
        println!("global_commit {}", xid);
        client
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
        client
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
        println!("global_report--{}", xid);
        Ok(GlobalStatus::Finished)
    }
}
