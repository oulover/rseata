use crate::resource::DefaultResourceManager;
use async_trait::async_trait;
use rseata_core::branch::branch_manager_inbound::BranchManagerInbound;
use rseata_core::branch::branch_manager_outbound::BranchManagerOutbound;
use rseata_core::branch::{BranchId, BranchStatus, BranchType};
use rseata_core::types::{ResourceId, Xid};

#[async_trait]
impl BranchManagerInbound for DefaultResourceManager {
    async fn branch_commit(
        &self,
        branch_type: BranchType,
        xid: Xid,
        branch_id: BranchId,
        resource_id: ResourceId,
        application_data: String,
    ) -> anyhow::Result<BranchStatus> {
        tracing::info!("BranchManagerInbound branch_commit------");

        // 使用 Option::take 来获取所有权而不是移动
        let mut branch_transaction = {
            let mut transactions = self.branch_transactions.write().await;
            transactions.remove(&branch_id)
        };

       let branch_status = if let Some(branch_transaction) = branch_transaction.take() {
           branch_transaction
                .branch_commit(
                    branch_type,
                    xid.clone(),
                    branch_id,
                    resource_id,
                    application_data.clone(),
                )
                .await?
        }else {
           tracing::error!("Branch commit failed: BranchId {} not exist", branch_id);
            BranchStatus::PhaseTwoCommitted
        };

        let _ = self
            .branch_report(
                branch_type,
                xid,
                branch_id,
                branch_status,
                application_data,
            )
            .await;
        Ok(branch_status)
    }

    async fn branch_rollback(
        &self,
        branch_type: BranchType,
        xid: Xid,
        branch_id: BranchId,
        resource_id: ResourceId,
        application_data: String,
    ) -> anyhow::Result<BranchStatus> {
        tracing::info!("BranchManagerInbound branch_rollback----------");
        let branch_transactions = {
            let mut t = self.branch_transactions.write().await;
            t.remove(&branch_id)
        };
        let branch_status = if let Some(branch_transactions) = branch_transactions {
            branch_transactions
                .branch_rollback(
                    branch_type,
                    xid.clone(),
                    branch_id,
                    resource_id,
                    application_data.clone(),
                )
                .await?
        }else {
            tracing::error!("Branch_rollback failed: BranchId {} not exist", branch_id);
            BranchStatus::PhaseTwoRollbacked
        };

        let _ = self
            .branch_report(
                branch_type,
                xid,
                branch_id,
                branch_status,
                application_data,
            )
            .await;
        Ok(branch_status)
    }
}
