use crate::error::TransactionError;

use crate::branch::{BranchId, BranchType};
use crate::session::defaults::default_branch_session::DefaultBranchSession;
use crate::session::global_session::GlobalSession;
use crate::session::session_storable::SessionStorable;
use crate::types::{GlobalStatus, Xid};
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DefaultGlobalSession {
    pub xid: Xid,
    pub transaction_id: u64,
    pub status: GlobalStatus,
    pub application_id: String,
    pub transaction_service_group: String,
    pub transaction_name: String,
    pub timeout_millis: u64,
    pub begin_time_millis: u64,
    pub application_data: Option<String>,
    pub lazy_load_branch: bool,
    pub active: bool,
    pub branch_sessions: VecDeque<DefaultBranchSession>,
    #[serde(skip)]
    pub lifecycle_listeners: Arc<RwLock<HashSet<String>>>,
}

impl DefaultGlobalSession {
    pub fn new(
        application_id: String,
        transaction_service_group: String,
        transaction_name: String,
        timeout_millis: u64,
        lazy_load_branch: bool,
    ) -> Self {
        let transaction_id = Uuid::new_v4().as_u128() as u64;
        let xid = format!("{:x}:{}", transaction_id, transaction_service_group);

        Self {
            xid: Xid::from(xid),
            transaction_id,
            status: GlobalStatus::Begin,
            application_id,
            transaction_service_group,
            transaction_name,
            timeout_millis,
            begin_time_millis: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            application_data: None,
            lazy_load_branch,
            active: true,
            branch_sessions: VecDeque::new(),
            lifecycle_listeners: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    // Business methods
    pub fn add_branch(&mut self, branch_session: DefaultBranchSession) -> bool {
        self.branch_sessions.push_back(branch_session);
        true
    }

    pub async fn remove_branch(&mut self, branch_session: &DefaultBranchSession) -> bool {
        if let Some(pos) = self
            .branch_sessions
            .iter()
            .position(|bs| bs.branch_id == branch_session.branch_id)
        {
            self.branch_sessions.remove(pos);
            true
        } else {
            false
        }
    }

    pub async fn remove_branch_by_id(&mut self, branch_id: BranchId) -> bool {
        if let Some(pos) = self
            .branch_sessions
            .iter()
            .position(|bs| bs.branch_id == branch_id)
        {
            self.branch_sessions.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn can_be_committed_async(&self) -> bool {
        self.branch_sessions
            .iter()
            .all(|branch| branch.can_be_committed_async())
    }

    pub async fn has_at_branch(&self) -> bool {
        self.branch_sessions
            .iter()
            .any(|branch| branch.branch_type == BranchType::AT)
    }

    pub fn is_saga(&self) -> bool {
        false
    }

    pub fn is_timeout(&self) -> bool {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        (current_time - self.begin_time_millis) > self.begin_time_millis * 1000
    }

    pub async fn get_branch(&self, branch_id: BranchId) -> Option<DefaultBranchSession> {
        self.branch_sessions
            .iter()
            .find(|branch| branch.branch_id == branch_id)
            .cloned()
    }

    pub async fn sorted_branches(&self) -> VecDeque<DefaultBranchSession> {
        self.branch_sessions.clone()
    }

    // pub async fn reverse_sorted_branches(&self) -> Vec<DefaultBranchSession> {
    //     let mut branches = self.sorted_branches().await;
    //     branches.reverse();
    //     branches
    // }

    pub async fn has_branch(&self) -> bool {
        !self.branch_sessions.is_empty()
    }

    pub async fn clean(&self) -> Result<(), TransactionError> {
        // Release global session lock
        // lock_manager.release_global_session_lock(self).await
        Ok(())
    }

    pub async fn close_and_clean(&mut self) -> Result<(), TransactionError> {
        // self.close().await?;
        if self.has_at_branch().await {
            self.clean().await?;
        }
        Ok(())
    }

    pub async fn add_session_lifecycle_listener(&self, listener_id: String) {
        self.lifecycle_listeners.write().await.insert(listener_id);
    }

    pub async fn remove_session_lifecycle_listener(&self, listener_id: &str) {
        self.lifecycle_listeners.write().await.remove(listener_id);
    }

    pub async fn async_commit(&self) -> Result<(), TransactionError> {
        // self.change_global_status(GlobalStatus::AsyncCommitting)
        //     .await
        Ok(())
    }
}

impl GlobalSession for DefaultGlobalSession {
    type BranchSession = DefaultBranchSession;

    fn xid(&self) -> &Xid {
        &self.xid
    }

    fn transaction_id(&self) -> u64 {
        self.transaction_id
    }

    fn status(&self) -> GlobalStatus {
        self.status.clone()
    }

    fn application_id(&self) -> &str {
        &self.application_id
    }

    fn transaction_service_group(&self) -> &str {
        &self.transaction_service_group
    }

    fn transaction_name(&self) -> &str {
        &self.transaction_name
    }

    fn timeout_millis(&self) -> u64 {
        self.timeout_millis
    }

    fn begin_time_millis(&self) -> u64 {
        self.begin_time_millis
    }

    fn application_data(&self) -> Option<&str> {
        self.application_data.as_deref()
    }

    fn lazy_load_branch(&self) -> bool {
        self.lazy_load_branch
    }

    fn active(&self) -> bool {
        self.active
    }

    fn branch_sessions(&self) -> Vec<Self::BranchSession> {
        vec![]
    }
}

// #[async_trait]
// impl SessionLifecycle for DefaultGlobalSession {
//     type BranchSession = DefaultBranchSession;
//
//     async fn begin(&mut self) -> Result<(), TransactionError> {
//         self.active = true;
//         // Notify session manager and listeners
//         Ok(())
//     }
//
//     async fn change_global_status(&self, status: GlobalStatus) -> Result<(), TransactionError> {
//         if status == GlobalStatus::Rollbacking || status == GlobalStatus::TimeoutRollbacking {
//             // Update lock status for all branches
//             // lock_manager.update_lock_status(&self.xid, LockStatus::Rollbacking).await?;
//         }
//
//         // Notify session manager
//         // session_manager.on_status_change(self, status).await?;
//
//         // Update status after successful notification
//         // self.status = status;
//
//         // Notify listeners
//         Ok(())
//     }
//
//     async fn change_branch_status(
//         &self,
//         branch_session: &DefaultBranchSession,
//         status: BranchStatus,
//     ) -> Result<(), TransactionError> {
//         // Notify session manager
//         // session_manager.on_branch_status_change(self, branch_session, status).await?;
//
//         // Notify listeners
//         Ok(())
//     }
//
//     async fn add_branch(&self, branch_session: DefaultBranchSession) -> Result<(), TransactionError> {
//         // Notify session manager
//         // session_manager.on_add_branch(self, &branch_session).await?;
//
//         // Notify listeners
//         if !cfg!(feature = "raft_mode") {
//             self.add_branch(branch_session);
//         }
//         Ok(())
//     }
//
//     async fn unlock_branch(&self, branch_session: &DefaultBranchSession) -> Result<(), TransactionError> {
//         // Don't unlock if global status is in committing states
//         if self.status != GlobalStatus::Committing
//             && self.status != GlobalStatus::CommitRetrying
//             && self.status != GlobalStatus::AsyncCommitting
//         {
//             branch_session.unlock().await?;
//         }
//         Ok(())
//     }
//
//     async fn remove_branch(&self, branch_session: &DefaultBranchSession) -> Result<(), TransactionError> {
//         // Notify session manager
//         // session_manager.on_remove_branch(self, branch_session).await?;
//
//         // Notify listeners
//
//         if !cfg!(feature = "raft_mode") {
//             self.remove_branch(branch_session);
//         }
//         Ok(())
//     }
//
//     async fn remove_and_unlock_branch(
//         &self,
//         branch_session: &DefaultBranchSession,
//     ) -> Result<(), TransactionError> {
//         self.unlock_branch(branch_session).await?;
//         self.remove_branch(branch_session);
//         Ok(())
//     }
//
//     fn is_active(&self) -> bool {
//         todo!()
//     }
//
//     async fn close(&mut self) -> Result<(), TransactionError> {
//         if self.active {
//             // Notify session manager
//             // session_manager.on_close(self).await?;
//
//             // Notify listeners
//             self.active = false;
//         }
//         Ok(())
//     }
//
//     async fn end(&self) -> Result<(), TransactionError> {
//         if GlobalStatus::is_two_phase_success(self.status) {
//             self.clean().await?;
//             // Notify session manager on success end
//             // session_manager.on_success_end(self).await?;
//         } else {
//             // Notify session manager on fail end
//             // session_manager.on_fail_end(self).await?;
//         }
//         Ok(())
//     }
// }

impl SessionStorable for DefaultGlobalSession {
    fn encode(&self) -> Result<Vec<u8>, TransactionError> {
        serde_json::to_vec(self).map_err(|e| TransactionError::new(e.to_string()))
    }

    fn decode(data: &[u8]) -> Result<Self, TransactionError> {
        serde_json::from_slice(data).map_err(|e| TransactionError::new(e.to_string()))
    }

    fn max_size(&self) -> usize {
        1
    }
}

impl std::fmt::Display for DefaultGlobalSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DefaultGlobalSession{{xid='{}', transactionId={}, status={:?}}}",
            self.xid, self.transaction_id, self.status
        )
    }
}
