use crate::coordinator::default_coordinator::DefaultCoordinator;
use async_trait::async_trait;
use rseata_core::branch::branch_manager_outbound::BranchManagerOutbound;
use rseata_core::branch::{BranchId, BranchStatus, BranchType};
use rseata_core::types::{ClientId, ResourceId, Xid};

#[async_trait]
impl BranchManagerOutbound for DefaultCoordinator {
    async fn branch_register(
        &self,
        branch_type: BranchType,
        resource_id: ResourceId,
        client_id: ClientId,
        xid: Xid,
        application_data: String,
        lock_keys: String,
    ) -> anyhow::Result<BranchId> {
        self.core
            .branch_register(
                branch_type,
                resource_id,
                client_id,
                xid,
                application_data,
                lock_keys,
            )
            .await
    }

    async fn branch_report(
        &self,
        branch_type: BranchType,
        xid: Xid,
        branch_id: BranchId,
        status: BranchStatus,
        application_data: String,
    ) -> anyhow::Result<()> {
        self.core
            .branch_report(branch_type, xid, branch_id, status, application_data)
            .await
    }

    async fn lock_query(
        &self,
        branch_type: BranchType,
        resource_id: ResourceId,
        xid: Xid,
        lock_keys: String,
    ) -> anyhow::Result<bool> {
        self.core
            .lock_query(branch_type, resource_id, xid, lock_keys)
            .await
    }
}
