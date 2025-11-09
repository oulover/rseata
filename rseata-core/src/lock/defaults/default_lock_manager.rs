use crate::branch::BranchId;
use crate::error::TransactionError;
use crate::lock::lock_manager::LockManager;
use crate::lock::locker::Locker;
use crate::lock::row_lock::RowLockData;
use crate::session::branch_session::BranchSession;
use crate::session::global_session::GlobalSession;
use crate::types::{ResourceId, Xid};
use async_trait::async_trait;
use std::sync::Arc;

/// 默认锁管理器实现
pub struct DefaultLockManager<L: Locker> {
    locker: Arc<L>,
}

impl<L: Locker> DefaultLockManager<L> {
    pub fn new(locker: Arc<L>) -> Self {
        Self { locker }
    }
}

#[async_trait]
impl<L: Locker> LockManager for DefaultLockManager<L> {
    type L = L;
    type RowLock = L::RowLock;

    async fn acquire_lock(
        &self,
        branch_session: &dyn BranchSession,
    ) -> Result<bool, TransactionError> {
        self.acquire_lock_with_options(branch_session, true, false)
            .await
    }

    async fn acquire_lock_with_options(
        &self,
        branch_session: &dyn BranchSession,
        auto_commit: bool,
        skip_check_lock: bool,
    ) -> Result<bool, TransactionError> {
        let row_locks = self.collect_row_locks(branch_session).await?;
        if row_locks.is_empty() {
            return Ok(true);
        }

        self.locker
            .acquire_lock_with_options(&row_locks, auto_commit, skip_check_lock)
            .await
    }

    async fn release_lock(
        &self,
        branch_session: &dyn BranchSession,
    ) -> Result<bool, TransactionError> {
        self.locker
            .release_lock_by_xid_branch_id(branch_session.xid(), branch_session.branch_id())
            .await
    }

    async fn release_global_session_lock<T: BranchSession + Send + Sync>(
        &self,
        global_session: &dyn GlobalSession<BranchSession = T>,
    ) -> Result<bool, TransactionError> {
        self.locker.release_lock_by_xid(global_session.xid()).await
    }

    async fn is_lockable(
        &self,
        xid: &Xid,
        resource_id: &ResourceId,
        transaction_id: u64,
        lock_key: &str,
    ) -> Result<bool, TransactionError> {
        // 解析 lock_key 并创建对应的行锁进行检查
        let row_locks = self
            .parse_lock_key(lock_key, resource_id, xid, transaction_id, None)
            .await?;
        self.locker.is_lockable(&row_locks).await
    }

    async fn clean_all_locks(&self) -> Result<(), TransactionError> {
        self.locker.clean_all_locks().await
    }

    async fn collect_row_locks(
        &self,
        branch_session: &dyn BranchSession,
    ) -> Result<Vec<Self::RowLock>, TransactionError> {
        if let Some(lock_key) = branch_session.lock_key() {
            self.parse_lock_key(
                lock_key,
                branch_session.resource_id().as_ref().unwrap(),
                branch_session.xid(),
                branch_session.transaction_id(),
                Some(branch_session.branch_id()),
            )
            .await
        } else {
            Ok(Vec::new())
        }
    }

    async fn update_lock_status(&self) -> Result<(), TransactionError> {
        // 实现锁状态更新逻辑
        Ok(())
    }
}

impl<L: Locker> DefaultLockManager<L> {
    async fn parse_lock_key_test(
        &self,
        lock_key: &str,
        resource_id: &ResourceId,
        xid: &Xid,
        transaction_id: u64,
        branch_id: Option<BranchId>,
    ) -> Result<Vec<L::RowLock>, TransactionError> {
        let mut row_locks = Vec::new();

        // 解析锁键格式: table1:pk1,pk2;table2:pk3,pk4
        for table_group in lock_key.split(';') {
            if let Some((table_name, pks_str)) = table_group.split_once(':') {
                for pk in pks_str.split(',') {
                    if !pk.trim().is_empty() {
                        let row_lock = L::RowLock::from(RowLockData {
                            xid: xid.clone(),
                            transaction_id,
                            branch_id,
                            resource_id: resource_id.clone(),
                            table_name: table_name.to_string(),
                            pk: pk.trim().to_string(),
                            row_key: None,
                            feature: None,
                        });
                        // 这里需要将 DefaultRowLock 转换为 L::RowLock
                        // 由于类型系统限制，这里简化处理
                        row_locks.push(row_lock);
                    }
                }
            }
        }

        Ok(row_locks)
    }

    async fn parse_lock_key(
        &self,
        lock_key: &str,
        resource_id: &ResourceId,
        xid: &Xid,
        transaction_id: u64,
        branch_id: Option<BranchId>,
    ) -> Result<Vec<L::RowLock>, TransactionError> {
        if lock_key.trim().is_empty() {
            return Ok(Vec::new());
        }

        let mut locks = Vec::new();
        let table_grouped_lock_keys: Vec<&str> = lock_key.split(';').collect();

        for table_grouped_lock_key in table_grouped_lock_keys {
            if table_grouped_lock_key.trim().is_empty() {
                continue;
            }

            // 分割表名和主键
            let parts: Vec<&str> = table_grouped_lock_key.splitn(2, ':').collect();
            if parts.len() != 2 {
                // 如果格式不正确，跳过这个分组
                continue;
            }

            let table_name = parts[0].trim();
            let merged_pks = parts[1].trim();

            if table_name.is_empty() || merged_pks.is_empty() {
                continue;
            }

            // 分割主键
            let pks: Vec<&str> = merged_pks.split(',').collect();
            if pks.is_empty() {
                continue;
            }

            for pk in pks {
                let pk = pk.trim();
                if !pk.is_empty() {
                    let row_lock = L::RowLock::from(RowLockData {
                        xid: xid.clone(),
                        transaction_id,
                        branch_id,
                        resource_id: resource_id.clone(),
                        table_name: table_name.to_string(),
                        pk: pk.trim().to_string(),
                        row_key: None,
                        feature: None,
                    });
                    locks.push(row_lock);
                }
            }
        }

        Ok(locks)
    }
}
