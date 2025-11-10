use async_trait::async_trait;
use rseata_core::branch::branch_manager_outbound::BranchManagerOutbound;
use rseata_core::branch::{BranchId, BranchStatus, BranchType};
use rseata_core::types::{ClientId, ResourceId, Xid};
use rseata_proto::rseata_proto::proto::{BranchRegisterRequest, BranchReportRequest, LockQueryRequest};
use crate::resource::DefaultResourceManager;

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