use crate::branch::BranchId;
use crate::error::TransactionError;
use crate::lock::LockStatus;
use crate::lock::defaults::default_row_lock::DefaultRowLock;
use crate::lock::locker::Locker;
use crate::lock::row_lock::RowLock;
use crate::types::{ResourceId, Xid};
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
struct LockEntry {
    xid: Xid,
    transaction_id: u64,
    branch_id: Option<BranchId>,
    status: LockStatus,
}

#[derive(Debug)]
pub struct MemoryLocker {
    locks: Arc<RwLock<HashMap<String, LockEntry>>>,
    xid_index: Arc<RwLock<HashMap<Xid, HashSet<String>>>>,
    branch_index: Arc<RwLock<HashMap<BranchId, HashSet<String>>>>,
}

impl MemoryLocker {
    pub fn new() -> Self {
        Self {
            locks: Arc::new(RwLock::new(HashMap::new())),
            xid_index: Arc::new(RwLock::new(HashMap::new())),
            branch_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn build_row_key(resource_id: &ResourceId, table_name: &str, pk: &str) -> String {
        format!("{}:{}:{}", resource_id, table_name, pk)
    }

    async fn add_to_index(&self, xid: &Xid, branch_id: Option<BranchId>, row_key: String) {
        {
            let mut xid_index = self.xid_index.write().await;
            xid_index
                .entry(xid.clone())
                .or_insert_with(HashSet::new)
                .insert(row_key.clone());
        }
        if let Some(branch_id) = branch_id {
            let mut branch_index = self.branch_index.write().await;
            branch_index
                .entry(branch_id)
                .or_insert_with(HashSet::new)
                .insert(row_key);
        }
    }

    async fn remove_from_index(&self, xid: &Xid, branch_id: Option<BranchId>, row_key: &str) {
        {
            let mut xid_index = self.xid_index.write().await;
            if let Some(keys) = xid_index.get_mut(xid) {
                keys.remove(row_key);
                if keys.is_empty() {
                    xid_index.remove(xid);
                }
            }
        }
        if let Some(branch_id) = branch_id {
            let mut branch_index = self.branch_index.write().await;
            if let Some(keys) = branch_index.get_mut(&branch_id) {
                keys.remove(row_key);
                if keys.is_empty() {
                    branch_index.remove(&branch_id);
                }
            }
        }
    }
}

#[async_trait]
impl Locker for MemoryLocker {
    type RowLock = DefaultRowLock;

    async fn acquire_lock(&self, row_locks: &[Self::RowLock]) -> Result<bool, TransactionError> {
        self.acquire_lock_with_options(row_locks, true, false).await
    }

    async fn acquire_lock_with_options(
        &self,
        row_locks: &[Self::RowLock],
        auto_commit: bool,
        skip_check_lock: bool,
    ) -> Result<bool, TransactionError> {
        if row_locks.is_empty() {
            return Ok(true);
        }

        let mut locks = self.locks.write().await;

        // 检查锁冲突
        if !skip_check_lock {
            for row_lock in row_locks {
                let row_key = Self::build_row_key(
                    row_lock.resource_id(),
                    row_lock.table_name(),
                    row_lock.pk(),
                );

                if let Some(existing_lock) = locks.get(&row_key) {
                    // 锁已被其他事务持有
                    if &existing_lock.xid != row_lock.xid() {
                        // 如果处于回滚状态且非自动提交，快速失败
                        if !auto_commit && existing_lock.status == LockStatus::Rollbacking {
                            return Err(TransactionError::new("Lock is rollbacking".to_string()));
                        }
                        return Ok(false);
                    }
                }
            }
        }

        // 获取锁
        let mut acquired_keys = Vec::new();
        for row_lock in row_locks {
            let row_key =
                Self::build_row_key(row_lock.resource_id(), row_lock.table_name(), row_lock.pk());

            let lock_entry = LockEntry {
                xid: row_lock.xid().clone(),
                transaction_id: row_lock.transaction_id(),
                branch_id: row_lock.branch_id(),
                status: LockStatus::Locked,
            };

            locks.insert(row_key.clone(), lock_entry);
            acquired_keys.push((row_lock.xid().clone(), row_lock.branch_id(), row_key));
        }

        // 释放写锁，避免死锁
        drop(locks);

        // 更新索引
        for (xid, branch_id, row_key) in acquired_keys {
            self.add_to_index(&xid, branch_id, row_key).await;
        }

        Ok(true)
    }

    async fn release_lock(&self, row_locks: &[Self::RowLock]) -> Result<bool, TransactionError> {
        let mut locks = self.locks.write().await;

        for row_lock in row_locks {
            let row_key =
                Self::build_row_key(row_lock.resource_id(), row_lock.table_name(), row_lock.pk());

            if locks.remove(&row_key).is_some() {
                self.remove_from_index(row_lock.xid(), row_lock.branch_id(), &row_key)
                    .await;
            }
        }

        Ok(true)
    }

    async fn release_lock_by_xid_branch_id(
        &self,
        xid: &Xid,
        branch_id: BranchId,
    ) -> Result<bool, TransactionError> {
        let branch_keys = {
            let branch_index = self.branch_index.read().await;
            branch_index
                .get(&branch_id)
                .map(|keys| keys.iter().cloned().collect::<Vec<_>>())
                .unwrap_or_default()
        };

        let mut locks = self.locks.write().await;
        for row_key in branch_keys {
            if let Some(lock_entry) = locks.get(&row_key) {
                if &lock_entry.xid == xid {
                    locks.remove(&row_key);
                    self.remove_from_index(xid, Some(branch_id), &row_key).await;
                }
            }
        }

        Ok(true)
    }

    async fn release_lock_by_xid(&self, xid: &Xid) -> Result<bool, TransactionError> {
        let xid_keys = {
            let xid_index = self.xid_index.read().await;
            xid_index
                .get(xid)
                .map(|keys| keys.iter().cloned().collect::<Vec<_>>())
                .unwrap_or_default()
        };

        let mut locks = self.locks.write().await;
        for row_key in xid_keys {
            locks.remove(&row_key);
        }

        // 清理索引
        {
            let mut xid_index = self.xid_index.write().await;
            xid_index.remove(xid);
        }

        // 从分支索引中清理
        {
            let mut branch_index = self.branch_index.write().await;
            for keys in branch_index.values_mut() {
                keys.retain(|key| locks.contains_key(key));
            }
            branch_index.retain(|_, keys| !keys.is_empty());
        }

        Ok(true)
    }

    async fn is_lockable(&self, row_locks: &[Self::RowLock]) -> Result<bool, TransactionError> {
        if row_locks.is_empty() {
            return Ok(true);
        }

        let xid = row_locks[0].xid();
        let locks = self.locks.read().await;

        for row_lock in row_locks {
            let row_key =
                Self::build_row_key(row_lock.resource_id(), row_lock.table_name(), row_lock.pk());

            if let Some(existing_lock) = locks.get(&row_key) {
                if &existing_lock.xid != xid {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    async fn clean_all_locks(&self) -> Result<(), TransactionError> {
        {
            let mut locks = self.locks.write().await;
            locks.clear();
        }
        {
            let mut xid_index = self.xid_index.write().await;
            xid_index.clear();
        }
        {
            let mut branch_index = self.branch_index.write().await;
            branch_index.clear();
        }
        Ok(())
    }

    async fn update_lock_status(
        &self,
        xid: &Xid,
        lock_status: LockStatus,
    ) -> Result<(), TransactionError> {
        let xid_keys = {
            let xid_index = self.xid_index.read().await;
            xid_index
                .get(xid)
                .map(|keys| keys.iter().cloned().collect::<Vec<_>>())
                .unwrap_or_default()
        };

        let mut locks = self.locks.write().await;
        for row_key in xid_keys {
            if let Some(lock_entry) = locks.get_mut(&row_key) {
                lock_entry.status = lock_status;
            }
        }

        Ok(())
    }
}

impl Default for MemoryLocker {
    fn default() -> Self {
        Self::new()
    }
}
