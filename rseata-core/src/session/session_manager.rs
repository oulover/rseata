use crate::branch::BranchStatus;
use crate::error::TransactionError;
use crate::session::branch_session::BranchSession;
use crate::session::global_session::GlobalSession;
use crate::session::session_condition::SessionCondition;
use crate::types::{GlobalStatus, Xid};
use async_trait::async_trait;
use std::fmt::Debug;

#[async_trait]
pub trait SessionManager: Send + Sync + Debug {
    type GlobalSession: GlobalSession + Send + Sync;
    type BranchSession: BranchSession + Send + Sync;
    async fn add_global_session(
        &self,
        session: &Self::GlobalSession,
    ) -> Result<(), TransactionError>;

    async fn find_global_session(&self, xid: &Xid) -> Option<Self::GlobalSession>;

    async fn find_global_session_with_branches(
        &self,
        xid: &Xid,
        with_branch_sessions: bool,
    ) -> Option<Self::GlobalSession>;

    async fn update_global_session_status(
        &self,
        global_session: &Self::GlobalSession,
        status: GlobalStatus,
    ) -> Result<GlobalStatus, TransactionError>;

    async fn remove_global_session(
        &self,
        session: &Self::GlobalSession,
    ) -> Result<(), TransactionError>;

    async fn add_branch_session(
        &self,
        global_session: &Self::GlobalSession,
        branch_session: &Self::BranchSession,
    ) -> Result<(), TransactionError>;

    async fn update_branch_session_status(
        &self,
        global_session: &Self::GlobalSession,
        branch_session: &Self::BranchSession,
        status: BranchStatus,
    ) -> Result<(), TransactionError>;

    async fn remove_branch_session(
        &self,
        global_session: &Self::GlobalSession,
        branch_session: &Self::BranchSession,
    ) -> Result<(), TransactionError>;

    async fn all_sessions(&self) -> Vec<Self::GlobalSession>;

    async fn find_global_sessions(&self, condition: &SessionCondition) -> Vec<Self::GlobalSession>;

    async fn destroy(&self);
}
