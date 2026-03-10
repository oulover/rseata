use serde::{Deserialize, Serialize};
use crate::branch::BranchId;
use crate::types::Xid;

/// Type of SQL operation that generated the undo log.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SQLType {
    INSERT,
    UPDATE,
    DELETE,
}

/// Represents a before or after image of a row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowImage {
    /// Column names
    pub columns: Vec<String>,
    /// Column values (as JSON strings)
    pub values: Vec<serde_json::Value>,
}

/// Undo log for a single row change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoLog {
    pub branch_id: BranchId,
    pub xid: Xid,
    pub table_name: String,
    pub sql_type: SQLType,
    pub before_image: Option<RowImage>,
    pub after_image: Option<RowImage>,
    pub log_created: i64, // timestamp
    pub log_modified: i64,
}

/// Manager for undo log storage and retrieval.
#[async_trait::async_trait]
pub trait UndoLogManager: Send + Sync {
    /// Store an undo log for the given branch.
    async fn add_undo_log(&self, undo_log: UndoLog) -> anyhow::Result<()>;

    /// Retrieve all undo logs for a given branch.
    async fn get_undo_logs(&self, branch_id: BranchId) -> anyhow::Result<Vec<UndoLog>>;

    /// Delete all undo logs for a given branch (after successful commit).
    async fn delete_undo_logs(&self, branch_id: BranchId) -> anyhow::Result<()>;

    /// Batch delete undo logs for multiple branches.
    async fn batch_delete_undo_logs(&self, branch_ids: Vec<BranchId>) -> anyhow::Result<()>;
}