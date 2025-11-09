use crate::error::TransactionError;
use crate::lock::locker::Locker;
use crate::lock::row_lock::RowLock;
use crate::session::branch_session::BranchSession;
use crate::session::global_session::GlobalSession;
use crate::types::{ResourceId, Xid};
use async_trait::async_trait;

#[async_trait]
pub trait LockManager: Send + Sync {
    type L: Locker<RowLock = Self::RowLock>;
    type RowLock: RowLock + Send + Sync;
    async fn acquire_lock(
        &self,
        branch_session: &dyn BranchSession,
    ) -> Result<bool, TransactionError>;
    async fn acquire_lock_with_options(
        &self,
        branch_session: &dyn BranchSession,
        auto_commit: bool,
        skip_check_lock: bool,
    ) -> Result<bool, TransactionError>;
    async fn release_lock(
        &self,
        branch_session: &dyn BranchSession,
    ) -> Result<bool, TransactionError>;
    async fn release_global_session_lock<T: BranchSession + Send + Sync>(
        &self,
        global_session: &dyn GlobalSession<BranchSession = T>,
    ) -> Result<bool, TransactionError>;
    async fn is_lockable(
        &self,
        xid: &Xid,
        resource_id: &ResourceId,
        transaction_id: u64,
        lock_key: &str,
    ) -> Result<bool, TransactionError>;
    async fn clean_all_locks(&self) -> Result<(), TransactionError>;
    async fn collect_row_locks(
        &self,
        branch_session: &dyn BranchSession,
    ) -> Result<Vec<Self::RowLock>, TransactionError>;
    async fn update_lock_status(&self) -> Result<(), TransactionError>;
}
