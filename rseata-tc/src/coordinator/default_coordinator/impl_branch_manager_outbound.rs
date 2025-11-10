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
        tracing::info!("Branch register :{branch_type:?}, {resource_id}, {client_id}, {xid}, {application_data}");
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
        branch_status: BranchStatus,
        application_data: String,
    ) -> anyhow::Result<()> {
        tracing::info!("Branch report :{branch_type:?}, {xid}, {branch_id}, {branch_status:?}");
        self.core
            .branch_report(branch_type, xid, branch_id, branch_status, application_data)
            .await
    }

    async fn lock_query(
        &self,
        branch_type: BranchType,
        resource_id: ResourceId,
        xid: Xid,
        lock_keys: String,
    ) -> anyhow::Result<bool> {
        tracing::info!("Lock query :{branch_type:?}, {resource_id}, {xid}, {lock_keys}");
        self.core
            .lock_query(branch_type, resource_id, xid, lock_keys)
            .await
    }
}
