use crate::coordinator::default_core_holder::DefaultCoreHolder;
use async_trait::async_trait;
use rseata_core::branch::BranchStatus;
use rseata_core::coordinator::transaction_coordinator_outbound::TransactionCoordinatorOutbound;
use rseata_core::event::event::TransactionEvent;
use rseata_core::event::event_publisher::EventPublisher;
use rseata_core::event::event_type::TransactionEventType;
use rseata_core::session::defaults::default_global_session::DefaultGlobalSession;
use rseata_core::session::session_manager::SessionManager;
use rseata_core::transaction::transaction_manager::TransactionManager;
use rseata_core::types::{GlobalStatus, Xid};
use std::sync::Arc;
use uuid::Uuid;
use rseata_core::coordinator::core_holder::CoreHolder;

#[async_trait]
impl TransactionManager for DefaultCoreHolder {
    async fn begin(
        &self,
        application_id: String,
        transaction_service_group: String,
        transaction_name: String,
        timeout_millis: u64,
    ) -> anyhow::Result<Xid> {
        let xid: Xid = Uuid::new_v4().to_string().into();
        let global_session = DefaultGlobalSession {
            xid: xid.clone(),
            transaction_id: Uuid::new_v4().as_u128() as u64,
            status: GlobalStatus::Begin,
            application_id,
            transaction_service_group,
            transaction_name,
            timeout_millis,
            begin_time_millis: 0,
            application_data: None,
            lazy_load_branch: false,
            active: false,
            branch_sessions: Default::default(),
            lifecycle_listeners: Arc::new(Default::default()),
        };

        self.event_publisher
            .publish(TransactionEvent {
                event_id: Uuid::new_v4().to_string(),
                timestamp: Default::default(),
                event_type: TransactionEventType::GlobalBegin {
                    xid: xid.clone(),
                    timeout_millis,
                },
                xid: Xid(String::new()),
                application_id: "".to_string(),
                transaction_name: "".to_string(),
                metadata: Default::default(),
            })
            .await;

        self.session_manager
            .add_global_session(&global_session)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        Ok(xid)
    }

    async fn commit(&self, xid: Xid) -> anyhow::Result<GlobalStatus> {
        let session = self
            .session_manager
            .find_global_session_with_branches(&xid, true)
            .await
            .ok_or_else(|| tonic::Status::cancelled(format!("no such global session {}", xid)))?;

        let can_commit = session
            .branch_sessions
            .iter()
            .all(|b| b.status == BranchStatus::PhaseOneDone);

        let mut can_rollback = session.branch_sessions.iter().all(|b| {
            b.status == BranchStatus::PhaseOneFailed || b.status == BranchStatus::PhaseOneTimeout
        });

        let mut global_status = self
            .session_manager
            .update_global_session_status(&session, GlobalStatus::Committing)
            .await?;
        if can_commit {
            for branch_session in session.branch_sessions.iter() {
                let result = self
                    .get_core(branch_session.branch_type)
                    .branch_commit(&session, branch_session)
                    .await;
                if result.is_err() {
                    can_rollback = true;
                }
            }
        }
        if can_rollback {
            global_status = self
                .session_manager
                .update_global_session_status(&session, GlobalStatus::Rollbacking)
                .await?;
            self.rollback(xid).await?;
        }
        Ok(global_status)
    }

    async fn rollback(&self, xid: Xid) -> anyhow::Result<GlobalStatus> {
        let session = self
            .session_manager
            .find_global_session_with_branches(&xid, true)
            .await
            .ok_or_else(|| tonic::Status::cancelled(format!("no such global session {}", xid)))?;

        let mut global_status = self
            .session_manager
            .update_global_session_status(&session, GlobalStatus::Rollbacking)
            .await?;
        for branch_session in session.branch_sessions.iter() {
            let result = self
                .get_core(branch_session.branch_type)
                .branch_rollback(&session, branch_session)
                .await;
            if result.is_err() {
                global_status = self
                    .session_manager
                    .update_global_session_status(&session, GlobalStatus::RollbackFailed)
                    .await?;
            }
        }
        Ok(global_status)
    }

    async fn get_status(&self, xid: Xid) -> anyhow::Result<GlobalStatus> {
        let session = self
            .session_manager
            .find_global_session_with_branches(&xid, true)
            .await
            .ok_or_else(|| tonic::Status::cancelled(format!("no such global session {}", xid)))?;
        Ok(session.status)
    }

    async fn global_report(
        &self,
        xid: &Xid,
        global_status: GlobalStatus,
    ) -> anyhow::Result<GlobalStatus> {
        let session = self
            .session_manager
            .find_global_session_with_branches(&xid, true)
            .await
            .ok_or_else(|| tonic::Status::cancelled(format!("no such global session {}", xid)))?;
        Ok(session.status)
    }
}
