use crate::branch::BranchStatus;
use crate::error::TransactionError;
use crate::session::defaults::default_branch_session::DefaultBranchSession;
use crate::session::defaults::default_global_session::DefaultGlobalSession;
use crate::session::global_session::GlobalSession;
use crate::session::session_condition::SessionCondition;
use crate::session::session_manager::SessionManager;
use crate::store::LogOperation;
use crate::store::transaction_store_manager::TransactionStoreManager;
use crate::types::{GlobalStatus, Xid};
use async_trait::async_trait;

#[derive(Debug)]
pub struct DefaultSessionManager {
    name: String,
    transaction_store_manager:
        Box<dyn TransactionStoreManager<GlobalSession = DefaultGlobalSession>>,
    rollback_failed_unlock_enable: bool,
}

impl DefaultSessionManager {
    pub fn new(
        name: String,
        transaction_store_manager: Box<
            dyn TransactionStoreManager<GlobalSession = DefaultGlobalSession>,
        >,
    ) -> Self {
        Self {
            name,
            transaction_store_manager,
            rollback_failed_unlock_enable: true, // Should be from config
        }
    }

    async fn write_session(
        &self,
        log_operation: LogOperation,
        session: &DefaultGlobalSession,
    ) -> Result<(), TransactionError> {
        self.transaction_store_manager
            .write_session(log_operation, session)
            .await
            .map_err(|e| TransactionError::ErrorInfo {
                info: String::from(e.to_string()),
            })?;
        Ok(())
    }
}

#[async_trait]
impl SessionManager for DefaultSessionManager {
    type GlobalSession = DefaultGlobalSession;
    type BranchSession = DefaultBranchSession;

    async fn add_global_session(
        &self,
        session: &Self::GlobalSession,
    ) -> Result<(), TransactionError> {
        self.write_session(LogOperation::GlobalUpdate, session)
            .await
    }

    async fn find_global_session(&self, xid: &Xid) -> Option<Self::GlobalSession> {
        self.transaction_store_manager.read_session(xid).await
    }

    async fn find_global_session_with_branches(
        &self,
        xid: &Xid,
        with_branch_sessions: bool,
    ) -> Option<Self::GlobalSession> {
        if with_branch_sessions {
            self.transaction_store_manager
                .read_session_with_branches(xid)
                .await
        } else {
            self.find_global_session(xid).await
        }
    }

    async fn update_global_session_status(
        &self,
        session: &Self::GlobalSession,
        status: GlobalStatus,
    ) -> Result<GlobalStatus, TransactionError> {
        if status == GlobalStatus::Rollbacking || status == GlobalStatus::TimeoutRollbacking {
            // Update lock status for all branches
            // for branch in session.branch_sessions() {
            //     // branch.set_lock_status(LockStatus::Rollbacking);
            // }
        }

        // session.set_status(status);
        self.write_session(LogOperation::GlobalUpdate, session)
            .await
            .map(|_| status)
    }

    async fn remove_global_session(
        &self,
        session: &Self::GlobalSession,
    ) -> Result<(), TransactionError> {
        self.write_session(LogOperation::GlobalRemove, session)
            .await
    }

    async fn add_branch_session(
        &self,
        global_session: &Self::GlobalSession,
        branch_session: &Self::BranchSession,
    ) -> Result<(), TransactionError> {
        let mut gs = self
            .find_global_session(global_session.xid())
            .await
            .ok_or_else(|| TransactionError::ErrorInfo {
                info: String::from("global_session"),
            })?;
        gs.add_branch(branch_session.clone());
        self.write_session(LogOperation::BranchAdd, &gs).await
    }

    async fn update_branch_session_status(
        &self,
        global_session: &Self::GlobalSession,
        branch_session: &Self::BranchSession,
        status: BranchStatus,
    ) -> Result<(), TransactionError> {
        let mut gs = self
            .find_global_session(global_session.xid())
            .await
            .ok_or_else(|| TransactionError::ErrorInfo {
                info: String::from("global_session"),
            })?;

        gs.branch_sessions.iter_mut().for_each(|e| {
            if e.branch_id == branch_session.branch_id {
                e.status = status;
            }
        });

        self.write_session(LogOperation::BranchUpdate, &gs).await
    }

    async fn remove_branch_session(
        &self,
        global_session: &Self::GlobalSession,
        branch_session: &Self::BranchSession,
    ) -> Result<(), TransactionError> {
        self.write_session(LogOperation::BranchRemove, global_session)
            .await
    }

    async fn all_sessions(&self) -> Vec<Self::GlobalSession> {
        vec![]
    }

    async fn find_global_sessions(&self, condition: &SessionCondition) -> Vec<Self::GlobalSession> {
        vec![]
    }

    async fn destroy(&self) {
        todo!()
    }
}
