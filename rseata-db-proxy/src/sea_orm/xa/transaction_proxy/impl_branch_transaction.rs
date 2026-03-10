use async_trait::async_trait;
use rseata_core::branch::branch_transaction::BranchTransaction;
use rseata_core::branch::{BranchId, BranchStatus, BranchType};
use rseata_core::types::{ResourceId, Xid};
use crate::sea_orm::xa::connection_proxy::XAConnectionProxy;
use sea_orm::{ConnectionTrait, Statement};
use std::sync::Arc;
use tokio::sync::Mutex;

/// XA Transaction implementation for distributed transactions
pub struct XABranchTransaction {
    connection_proxy: Arc<Mutex<XAConnectionProxy>>,
}

impl XABranchTransaction {
    pub fn new(connection_proxy: XAConnectionProxy) -> Self {
        Self {
            connection_proxy: Arc::new(Mutex::new(connection_proxy)),
        }
    }
}

#[async_trait]
impl BranchTransaction for XABranchTransaction {
    async fn branch_commit(
        &self,
        branch_type: BranchType,
        xid: Xid,
        branch_id: BranchId,
        resource_id: ResourceId,
        application_data: String,
    ) -> anyhow::Result<BranchStatus> {
        let mut conn = self.connection_proxy.lock().await;

        // 1. XA END: Disassociate the transaction from the connection
        let xa_end_sql = format!("XA END '{}'", format_xa_branch_xid(&xid.0, branch_id.0));
        conn.execute_unprepared(&xa_end_sql).await
            .map_err(|e| anyhow::anyhow!("XA END failed: {}", e))?;

        // 2. XA PREPARE: Prepare the transaction branch
        let xa_prepare_sql = format!("XA PREPARE '{}'", format_xa_branch_xid(&xid.0, branch_id.0));
        match conn.execute_unprepared(&xa_prepare_sql).await {
            Ok(_) => {
                // 3. XA COMMIT: Commit the prepared transaction
                let xa_commit_sql = format!("XA COMMIT '{}'", format_xa_branch_xid(&xid.0, branch_id.0));
                conn.execute_unprepared(&xa_commit_sql).await
                    .map_err(|e| anyhow::anyhow!("XA COMMIT failed: {}", e))?;

                tracing::info!("XA branch {} committed for xid {} and resource {}",
                              branch_id.0, xid.0, resource_id.0);
                Ok(BranchStatus::PhaseTwoCommitted)
            }
            Err(e) => {
                tracing::error!("XA PREPARE failed for branch {}, xid {}: {}", branch_id.0, xid.0, e);
                Ok(BranchStatus::PhaseTwoCommitFailedUnretryable)
            }
        }
    }

    async fn branch_rollback(
        &self,
        branch_type: BranchType,
        xid: Xid,
        branch_id: BranchId,
        resource_id: ResourceId,
        application_data: String,
    ) -> anyhow::Result<BranchStatus> {
        let mut conn = self.connection_proxy.lock().await;

        // XA END: Disassociate the transaction from the connection (if not already done)
        let xa_end_sql = format!("XA END '{}'", format_xa_branch_xid(&xid.0, branch_id.0));
        let _ = conn.execute_unprepared(&xa_end_sql).await; // Ignore errors for END as it might already be ended

        // XA ROLLBACK: Rollback the transaction branch
        let xa_rollback_sql = format!("XA ROLLBACK '{}'", format_xa_branch_xid(&xid.0, branch_id.0));
        conn.execute_unprepared(&xa_rollback_sql).await
            .map_err(|e| anyhow::anyhow!("XA ROLLBACK failed: {}", e))?;

        tracing::info!("XA branch {} rolled back for xid {} and resource {}",
                      branch_id.0, xid.0, resource_id.0);
        Ok(BranchStatus::PhaseTwoRollbacked)
    }
}

/// Format the XA transaction XID combining global XID and branch ID
fn format_xa_branch_xid(global_xid: &str, branch_id: u64) -> String {
    format!("{}_{}", global_xid, branch_id)
}

impl XAConnectionProxy {
    /// Start an XA transaction branch
    pub async fn xa_start_branch(&self, global_xid: &str, branch_id: u64) -> anyhow::Result<()> {
        let xa_xid = format_xa_branch_xid(global_xid, branch_id);
        let sql = format!("XA START '{}'", xa_xid);

        self.execute_unprepared(&sql).await
            .map_err(|e| anyhow::anyhow!("XA START failed: {}", e))?;

        Ok(())
    }

    /// End the current XA transaction branch
    pub async fn xa_end_branch(&self, global_xid: &str, branch_id: u64) -> anyhow::Result<()> {
        let xa_xid = format_xa_branch_xid(global_xid, branch_id);
        let sql = format!("XA END '{}'", xa_xid);

        self.execute_unprepared(&sql).await
            .map_err(|e| anyhow::anyhow!("XA END failed: {}", e))?;

        Ok(())
    }

    /// Prepare the XA transaction branch
    pub async fn xa_prepare_branch(&self, global_xid: &str, branch_id: u64) -> anyhow::Result<()> {
        let xa_xid = format_xa_branch_xid(global_xid, branch_id);
        let sql = format!("XA PREPARE '{}'", xa_xid);

        self.execute_unprepared(&sql).await
            .map_err(|e| anyhow::anyhow!("XA PREPARE failed: {}", e))?;

        Ok(())
    }

    /// Commit the XA transaction branch
    pub async fn xa_commit_branch(&self, global_xid: &str, branch_id: u64) -> anyhow::Result<()> {
        let xa_xid = format_xa_branch_xid(global_xid, branch_id);
        let sql = format!("XA COMMIT '{}'", xa_xid);

        self.execute_unprepared(&sql).await
            .map_err(|e| anyhow::anyhow!("XA COMMIT failed: {}", e))?;

        Ok(())
    }

    /// Rollback the XA transaction branch
    pub async fn xa_rollback_branch(&self, global_xid: &str, branch_id: u64) -> anyhow::Result<()> {
        let xa_xid = format_xa_branch_xid(global_xid, branch_id);
        let sql = format!("XA ROLLBACK '{}'", xa_xid);

        self.execute_unprepared(&sql).await
            .map_err(|e| anyhow::anyhow!("XA ROLLBACK failed: {}", e))?;

        Ok(())
    }
}