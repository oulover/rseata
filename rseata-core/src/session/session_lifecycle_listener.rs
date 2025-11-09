use crate::branch::BranchStatus;
use crate::error::TransactionError;
use crate::session::branch_session::BranchSession;
use crate::session::global_session::GlobalSession;
use crate::types::GlobalStatus;
use async_trait::async_trait;

#[async_trait]
pub trait SessionLifecycleListener: Send + Sync {
    type GlobalSession: GlobalSession + Send + Sync;
    type BranchSession: BranchSession + Send + Sync;
    async fn on_begin(&self, global_session: &Self::GlobalSession) -> Result<(), TransactionError>;
    async fn on_status_change(
        &self,
        global_session: &Self::GlobalSession,
        status: GlobalStatus,
    ) -> Result<(), TransactionError>;
    async fn on_branch_status_change(
        &self,
        global_session: &Self::GlobalSession,
        branch_session: &Self::BranchSession,
        status: BranchStatus,
    ) -> Result<(), TransactionError>;
    async fn on_add_branch(
        &self,
        global_session: &Self::GlobalSession,
        branch_session: &Self::BranchSession,
    ) -> Result<(), TransactionError>;
    async fn on_remove_branch(
        &self,
        global_session: &Self::GlobalSession,
        branch_session: &Self::BranchSession,
    ) -> Result<(), TransactionError>;
    async fn on_close(&self, global_session: &Self::GlobalSession) -> Result<(), TransactionError>;
    async fn on_success_end(
        &self,
        global_session: &Self::GlobalSession,
    ) -> Result<(), TransactionError>;
    async fn on_fail_end(
        &self,
        global_session: &Self::GlobalSession,
    ) -> Result<(), TransactionError>;
}
