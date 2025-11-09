pub mod memery_transaction_store_manager;
pub mod transaction_store_manager;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StoreMode {
    File,
    Db,
    Redis,
    Raft,
    Memory,
}

#[derive(Debug, Clone, Copy)]
pub enum LogOperation {
    GlobalAdd,
    GlobalUpdate,
    GlobalRemove,
    BranchAdd,
    BranchUpdate,
    BranchRemove,
}

#[derive(Debug, Clone)]
pub struct StoreConfig {
    pub session_mode: StoreMode,
    pub max_global_session_size: usize,
    pub max_branch_session_size: usize,
    pub file_store_dir: String,
    pub redis_host: String,
    pub redis_port: u16,
    pub db_url: String,
}

impl Default for StoreConfig {
    fn default() -> Self {
        Self {
            session_mode: StoreMode::File,
            max_global_session_size: 64 * 1024, // 64KB
            max_branch_session_size: 32 * 1024, // 32KB
            file_store_dir: "session_data".to_string(),
            redis_host: "localhost".to_string(),
            redis_port: 6379,
            db_url: "".to_string(),
        }
    }
}

impl StoreConfig {
    pub fn session_mode() -> StoreMode {
        // In real implementation, this would read from configuration
        StoreMode::File
    }

    pub fn get_max_global_session_size() -> usize {
        64 * 1024 // 64KB
    }

    pub fn get_max_branch_session_size() -> usize {
        32 * 1024 // 32KB
    }
}
