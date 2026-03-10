pub mod branch_session;
pub mod defaults;
pub mod global_session;
pub mod session_condition;
pub mod session_helper;
pub mod session_life_cycle;
mod session_lifecycle_listener;
pub mod session_manager;
pub mod session_storable;

use std::sync::RwLock;

use crate::branch::BranchId;
use crate::types::Xid;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

#[derive(Debug)]
pub struct ClientSession {
    pub transaction_name: String,
    xid: RwLock<Option<Xid>>,
    is_global_tx_started: AtomicBool,
    rm: RwLock<Vec<String>>,
    branch_id: AtomicU64,
    branch_undo_logs: tokio::sync::RwLock<Vec<String>>,
    branch_lock_keys: tokio::sync::RwLock<Option<String>>,
}

impl ClientSession {
    pub fn new(transaction_name: String) -> Self {
        Self {
            transaction_name,
            xid: RwLock::new(None),
            rm: RwLock::new(Vec::new()),
            is_global_tx_started: AtomicBool::new(false),
            branch_id: AtomicU64::new(0),
            branch_undo_logs: tokio::sync::RwLock::new(Vec::new()),
            branch_lock_keys: tokio::sync::RwLock::new(None),
        }
    }

    pub fn begin_global_transaction(&self, xid: Xid) -> Result<(), anyhow::Error> {
        let mut xid_guard = self.xid.write().unwrap();
        if xid_guard.is_none() {
            *xid_guard = Some(xid);
            self.is_global_tx_started.store(true, Ordering::Release);
            Ok(())
        } else {
            Err(anyhow::Error::msg("Global transaction already started"))
        }
    }

    pub fn get_xid(&self) -> Option<Xid> {
        let guard = self.xid.read().unwrap();
        guard.clone()
    }

    pub fn is_global_tx_started(&self) -> bool {
        self.is_global_tx_started.load(Ordering::Acquire)
    }

    pub fn set_branch_id(&self, branch_id: BranchId) {
        self.branch_id.store(branch_id.into(), Ordering::Release);
    }
    pub fn get_branch_id(&self) -> BranchId {
        self.branch_id.load(Ordering::Acquire).into()
    }

    pub async fn init_branch(&self) {
        if self.is_global_tx_started() {
            {
                let mut undo_logs = self.branch_undo_logs.write().await;
                undo_logs.clear();
                self.branch_id.store(0, Ordering::Release);
            }

            {
                let mut branch_lock_keys = self.branch_lock_keys.write().await;
                *branch_lock_keys = None;
            }
        }
    }

    pub async fn set_branch_lock_keys(&self, branch_lock_keys: String) {
        let branch_lock_keys = branch_lock_keys.trim();
        if !branch_lock_keys.is_empty() {
            let mut lock = self.branch_lock_keys.write().await;
            *lock = Some(branch_lock_keys.to_string());
        }
    }

    pub async fn get_branch_lock_keys(&self) -> Option<String> {
        self.branch_lock_keys.read().await.clone()
    }
}
