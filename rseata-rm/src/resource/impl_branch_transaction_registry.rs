use crate::resource::DefaultResourceManager;
use async_trait::async_trait;
use rseata_core::branch::branch_manager_outbound::BranchManagerOutbound;
use rseata_core::branch::branch_transaction::{BranchTransaction, BranchTransactionRegistry};
use rseata_core::branch::{BranchId, BranchType};
use rseata_core::types::{ClientId, ResourceId, Xid};

#[async_trait]
impl BranchTransactionRegistry for DefaultResourceManager {
    async fn branch_transaction_registry(
        &self,
        branch_type: BranchType,
        resource_id: ResourceId,
        client_id: ClientId,
        xid: Xid,
        application_data: String,
        lock_keys: String,
        branch_transaction: Box<dyn BranchTransaction>,
    ) -> anyhow::Result<BranchId> {
        let branch_id = self
            .branch_register(
                branch_type,
                resource_id,
                client_id,
                xid,
                application_data,
                lock_keys,
            )
            .await?;
        self.branch_transactions
            .write()
            .await
            .insert(branch_id.clone(), branch_transaction);
        Ok(branch_id)
    }
}
