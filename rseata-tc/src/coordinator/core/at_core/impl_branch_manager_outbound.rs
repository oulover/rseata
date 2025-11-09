use crate::coordinator::core::at_core::ATCore;
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
impl BranchManagerOutbound for ATCore {
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
        self.session_manager
            .add_branch_session(
                &global_session,
                &DefaultBranchSession {
                    xid,
                    transaction_id: global_session.transaction_id,
                    branch_id,
                    resource_group_id: None,
                    resource_id: Some(resource_id),
                    lock_key: None,
                    branch_type,
                    status: BranchStatus::Registered,
                    client_id,
                    application_data: None,
                    lock_status: LockStatus::Locked,
                    lock_holder: Default::default(),
                },
            )
            .await?;

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
        Ok(())
    }

    async fn lock_query(
        &self,
        branch_type: BranchType,
        resource_id: ResourceId,
        xid: Xid,
        lock_keys: String,
    ) -> anyhow::Result<bool> {
        tracing::debug!("---------------lock_query---------");
        let global_session = self
            .session_manager
            .find_global_session(&xid)
            .await
            .ok_or_else(|| {
                tonic::Status::invalid_argument(format!("no such global session {}", xid))
            })?;
        let r = self
            .lock_manager
            .is_lockable(
                &xid,
                &resource_id,
                global_session.transaction_id,
                lock_keys.as_str(),
            )
            .await?;
        tracing::debug!("---------------lock_query---------{}", r);
        Ok(r)
    }
}
