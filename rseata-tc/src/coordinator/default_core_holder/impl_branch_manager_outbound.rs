
use async_trait::async_trait;
use rseata_core::branch::branch_manager_outbound::BranchManagerOutbound;
use rseata_core::branch::{BranchId, BranchStatus, BranchType};
use rseata_core::coordinator::core_holder::CoreHolder;
use rseata_core::types::{ClientId, ResourceId, Xid};
use crate::coordinator::default_core_holder::DefaultCoreHolder;

#[async_trait]
impl BranchManagerOutbound for DefaultCoreHolder {
    async fn branch_register(
        &self,
        branch_type: BranchType,
        resource_id: ResourceId,
        client_id: ClientId,
        xid: Xid,
        application_data: String,
        lock_keys: String,
    ) -> anyhow::Result<BranchId> {
        self.get_core(branch_type)
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
        self.get_core(branch_type)
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
        self.get_core(branch_type)
            .lock_query(branch_type, resource_id, xid, lock_keys)
            .await
    }
}
