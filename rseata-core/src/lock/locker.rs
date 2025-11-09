use crate::branch::BranchId;
use crate::error::TransactionError;
use crate::lock::LockStatus;
use crate::lock::row_lock::RowLock;
use crate::types::Xid;
use async_trait::async_trait;

#[async_trait]
pub trait Locker: Send + Sync {
    type RowLock: RowLock + Send + Sync;
    async fn acquire_lock(&self, row_locks: &[Self::RowLock]) -> Result<bool, TransactionError>;
    async fn acquire_lock_with_options(
        &self,
        row_locks: &[Self::RowLock],
        auto_commit: bool,
        skip_check_lock: bool,
    ) -> Result<bool, TransactionError>;
    async fn release_lock(&self, row_locks: &[Self::RowLock]) -> Result<bool, TransactionError>;
    async fn release_lock_by_xid_branch_id(
        &self,
        xid: &Xid,
        branch_id: BranchId,
    ) -> Result<bool, TransactionError>;
    async fn release_lock_by_xid(&self, xid: &Xid) -> Result<bool, TransactionError>;
    async fn is_lockable(&self, row_locks: &[Self::RowLock]) -> Result<bool, TransactionError>;
    async fn clean_all_locks(&self) -> Result<(), TransactionError>;
    async fn update_lock_status(
        &self,
        xid: &Xid,
        lock_status: LockStatus,
    ) -> Result<(), TransactionError>;
}
