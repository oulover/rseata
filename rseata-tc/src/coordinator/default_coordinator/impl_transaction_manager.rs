use crate::coordinator::default_coordinator::DefaultCoordinator;
use async_trait::async_trait;
use rseata_core::transaction::transaction_manager::TransactionManager;
use rseata_core::types::{GlobalStatus, Xid};

#[async_trait]
impl TransactionManager for DefaultCoordinator {
    async fn begin(
        &self,
        application_id: String,
        transaction_service_group: String,
        transaction_name: String,
        timeout_millis: u64,
    ) -> anyhow::Result<Xid> {
        self.core
            .begin(
                application_id,
                transaction_service_group,
                transaction_name,
                timeout_millis,
            )
            .await
    }

    async fn commit(&self, xid: Xid) -> anyhow::Result<GlobalStatus> {
        self.core.commit(xid).await
    }

    async fn rollback(&self, xid: Xid) -> anyhow::Result<GlobalStatus> {
        self.core.rollback(xid).await
    }

    async fn get_status(&self, xid: Xid) -> anyhow::Result<GlobalStatus> {
        self.core.get_status(xid).await
    }

    async fn global_report(
        &self,
        xid: &Xid,
        global_status: GlobalStatus,
    ) -> anyhow::Result<GlobalStatus> {
        self.core.global_report(xid, global_status).await
    }
}
