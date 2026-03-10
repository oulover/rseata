use crate::coordinator::core::xa_core::XACore;
use async_trait::async_trait;
use rseata_core::branch::branch_manager_outbound::BranchManagerOutbound;
use rseata_core::branch::{BranchId, BranchStatus, BranchType};
use rseata_core::lock::LockStatus;
use rseata_core::lock::lock_manager::LockManager;
use rseata_core::session::defaults::default_branch_session::DefaultBranchSession;
use rseata_core::session::session_manager::SessionManager;
use rseata_core::types::{ClientId, ResourceId, Xid};
use uuid::Uuid;

#[async_trait]
impl BranchManagerOutbound for XACore {
    async fn branch_register(
        &self,
        branch_type: BranchType,
        resource_id: ResourceId,
        client_id: ClientId,
        xid: Xid,
        application_data: String,
        lock_keys: String,
    ) -> anyhow::Result<BranchId> {
        let global_session = self
            .session_manager
            .find_global_session(&xid)
            .await
            .ok_or_else(|| {
                tonic::Status::invalid_argument(format!("no such global session {}", xid))
            })?;

        let branch_id = BranchId::from(Uuid::new_v4().as_u128() as u64);

        let resource_id_for_log = resource_id.clone();
        let xid_for_log = xid.clone();

        // For XA mode, register the branch with appropriate initial status
        self.session_manager
            .add_branch_session(
                &global_session,
                &DefaultBranchSession {
                    xid,
                    transaction_id: global_session.transaction_id,
                    branch_id,
                    resource_group_id: None,
                    resource_id: Some(resource_id),
                    lock_key: Some(lock_keys),
                    branch_type,
                    status: BranchStatus::Registered, // Initially registered
                    client_id,
                    application_data: Some(application_data),
                    lock_status: LockStatus::Released, // In XA, locks are managed differently
                    lock_holder: Default::default(),
                },
            )
            .await?;

        tracing::info!("XA branch {} registered for xid {} and resource {}", branch_id.0, xid_for_log.0, resource_id_for_log.0);

        Ok(branch_id)
    }

    async fn branch_report(
        &self,
        branch_type: BranchType,
        xid: Xid,
        branch_id: BranchId,
        status: BranchStatus,
        application_data: String,
    ) -> anyhow::Result<()> {
        let global_session = self
            .session_manager
            .find_global_session(&xid)
            .await
            .ok_or_else(|| {
                tonic::Status::invalid_argument(format!("no such global session {}", xid))
            })?;

        let branch_session = global_session.get_branch(branch_id).await.ok_or_else(|| {
            tonic::Status::invalid_argument(format!("no such branch session {}", branch_id))
        })?;

        self.session_manager
            .update_branch_session_status(&global_session, &branch_session, status)
            .await?;

        tracing::info!("XA branch {} reported status {:?} for xid {}", branch_id.0, status, xid.0);

        Ok(())
    }

    async fn lock_query(
        &self,
        branch_type: BranchType,
        resource_id: ResourceId,
        xid: Xid,
        lock_keys: String,
    ) -> anyhow::Result<bool> {
        tracing::debug!("XA lock_query for resource {} and xid {}, lock_keys: {}", resource_id.0, xid.0, lock_keys);

        // For XA mode, the locking happens at the database level during XA operations
        // So we generally allow the lock query to pass
        // In XA, the database handles row-level locking during XA operations
        Ok(true)
    }
}