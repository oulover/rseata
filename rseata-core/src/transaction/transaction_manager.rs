use crate::types::{GlobalStatus, Xid};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait TransactionManager {
    async fn begin(
        &self,
        application_id: String,
        transaction_service_group: String,
        transaction_name: String,
        timeout_millis: u64,
    ) -> Result<Xid>;
    async fn commit(&self, xid: Xid) -> Result<GlobalStatus>;
    async fn rollback(&self, xid: Xid) -> Result<GlobalStatus>;
    async fn get_status(&self, xid: Xid) -> Result<GlobalStatus>;
    async fn global_report(&self, xid: &Xid, global_status: GlobalStatus) -> Result<GlobalStatus>;
}
