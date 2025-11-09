use crate::branch::BranchStatus;
use crate::error::TransactionError;
use crate::session::branch_session::BranchSession;
use crate::session::global_session::GlobalSession;
use async_trait::async_trait;

/// send outbound request to RM.
#[async_trait]
pub trait TransactionCoordinatorOutbound {
    type GlobalSession: GlobalSession + Send + Sync;
    type BranchSession: BranchSession + Send + Sync;
    async fn branch_commit(
        &self,
        global_session: &Self::GlobalSession,
        branch_session: &Self::BranchSession,
    ) -> Result<BranchStatus, TransactionError>;

    async fn branch_rollback(
        &self,
        global_session: &Self::GlobalSession,
        branch_session: &Self::BranchSession,
    ) -> Result<BranchStatus, TransactionError>;
    async fn branch_delete(
        &self,
        global_session: &Self::GlobalSession,
        branch_session: &Self::BranchSession,
    ) -> Result<BranchStatus, TransactionError>;
}
