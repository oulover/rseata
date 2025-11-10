use crate::coordinator::default_core_holder::DefaultCoreHolder;
use async_trait::async_trait;
use rseata_core::branch::BranchStatus;
use rseata_core::coordinator::core_holder::CoreHolder;
use rseata_core::event::event::TransactionEvent;
use rseata_core::event::event_publisher::EventPublisher;
use rseata_core::event::event_type::TransactionEventType;
use rseata_core::session::defaults::default_global_session::DefaultGlobalSession;
use rseata_core::session::session_manager::SessionManager;
use rseata_core::transaction::transaction_manager::TransactionManager;
use rseata_core::types::{GlobalStatus, Xid};
use std::sync::Arc;
use uuid::Uuid;

#[async_trait]
impl TransactionManager for DefaultCoreHolder {
    async fn begin(
        &self,
        application_id: String,
        transaction_service_group: String,
        transaction_name: String,
        timeout_millis: u64,
    ) -> anyhow::Result<Xid> {
        tracing::info!("Begin : {application_id},{transaction_service_group},{transaction_name}");
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
        tracing::info!("Commit : {xid}");
        let session = self
            .session_manager
            .find_global_session_with_branches(&xid, true)
            .await
            .ok_or_else(|| tonic::Status::cancelled(format!("no such global session {}", xid)))?;

        let can_commit = session
            .branch_sessions
            .iter()
            .all(|b| b.status == BranchStatus::PhaseOneDone);

        let can_rollback = session.branch_sessions.iter().all(|b| {
            b.status == BranchStatus::PhaseOneFailed || b.status == BranchStatus::PhaseOneTimeout
        });

        let mut global_status = self
            .session_manager
            .update_global_session_status(&session, GlobalStatus::Committing)
            .await?;

        if can_commit {
            let commit_futures: Vec<_> = session
                .branch_sessions
                .iter()
                .map(|branch_session| {
                    let core = self.get_core(branch_session.branch_type);
                    let session_ref = &session;
                    async move { core.branch_commit(session_ref, branch_session).await }
                })
                .collect();
            use futures::future::try_join_all;
            match try_join_all(commit_futures).await {
                Ok(_) => {
                    // 所有分支提交成功
                    global_status = self
                        .session_manager
                        .update_global_session_status(&session, GlobalStatus::Committed)
                        .await?;
                }
                Err(_) => {
                    // 有分支提交失败，开始回滚
                    global_status = self
                        .session_manager
                        .update_global_session_status(&session, GlobalStatus::Rollbacking)
                        .await?;
                    self.rollback(xid).await?;
                }
            }
        } else if can_rollback {
            // 直接回滚
            global_status = self
                .session_manager
                .update_global_session_status(&session, GlobalStatus::Rollbacking)
                .await?;
            self.rollback(xid).await?;
        }

        Ok(global_status)
    }

    async fn rollback(&self, xid: Xid) -> anyhow::Result<GlobalStatus> {
        tracing::info!("Rollback : {xid}");
        let session = self
            .session_manager
            .find_global_session_with_branches(&xid, true)
            .await
            .ok_or_else(|| tonic::Status::cancelled(format!("no such global session {}", xid)))?;

        let mut global_status = self
            .session_manager
            .update_global_session_status(&session, GlobalStatus::Rollbacking)
            .await?;

        let rollback_futures: Vec<_> = session
            .branch_sessions
            .iter()
            .map(|branch_session| {
                let core = self.get_core(branch_session.branch_type);
                let session_ref = &session;
                async move {
                    core.branch_rollback(session_ref, branch_session)
                        .await
                        .map_err(|e| {
                            tracing::error!("Branch rollback failed: {:?}", e);
                            e
                        })
                }
            })
            .collect();
        use futures::future::join_all;
        let results = join_all(rollback_futures).await;
        if results.iter().any(|r| r.is_err()) {
            global_status = self
                .session_manager
                .update_global_session_status(&session, GlobalStatus::RollbackFailed)
                .await?;
        }

        Ok(global_status)
    }

    async fn get_status(&self, xid: Xid) -> anyhow::Result<GlobalStatus> {
        tracing::info!("Get status : {xid}");
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
        tracing::info!("Global report : {xid}, {global_status:?}");
        let session = self
            .session_manager
            .find_global_session_with_branches(&xid, true)
            .await
            .ok_or_else(|| tonic::Status::cancelled(format!("no such global session {}", xid)))?;
        Ok(session.status)
    }
}
