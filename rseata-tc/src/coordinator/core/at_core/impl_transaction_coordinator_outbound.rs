use crate::coordinator::core::at_core::ATCore;
use async_trait::async_trait;
use rseata_core::branch::BranchStatus;
use rseata_core::coordinator::transaction_coordinator_outbound::TransactionCoordinatorOutbound;
use rseata_core::error::TransactionError;
use rseata_core::event::event_type::TransactionEventType::BranchRollback;
use rseata_core::session::defaults::default_branch_session::DefaultBranchSession;
use rseata_core::session::defaults::default_global_session::DefaultGlobalSession;
use rseata_proto::rseata_proto::proto::{
    BranchCommitInstruction, BranchRollbackInstruction, ResourceInstruction, resource_instruction,
};

#[async_trait]
impl TransactionCoordinatorOutbound for ATCore {
    type GlobalSession = DefaultGlobalSession;
    type BranchSession = DefaultBranchSession;

    async fn branch_commit(
        &self,
        global_session: &Self::GlobalSession,
        branch_session: &Self::BranchSession,
    ) -> Result<BranchStatus, TransactionError> {
        tracing::debug!(
            "TransactionCoordinatorOutbound branch_commit---{:?}",
            branch_session
        );

        let resource_id = branch_session.resource_id.clone().unwrap();
        let sender = {
            self.resources
                .read()
                .await
                .get(&resource_id)
                .ok_or_else(|| TransactionError::new(String::from("resource not found")))?
                .iter()
                .find(|rs| rs.resource.client_id == branch_session.client_id)
                .ok_or_else(|| TransactionError::new(String::from("client_id not found")))?
                .response_tx
                .clone()
        };

        sender
            .send(Ok(ResourceInstruction {
                instruction_id: 0,
                instruction: Some(resource_instruction::Instruction::Commit(
                    BranchCommitInstruction {
                        branch_type: branch_session.branch_type.into(),
                        xid: branch_session.xid.to_string(),
                        branch_id: branch_session.branch_id.into(),
                        resource_id: resource_id.0,
                        application_data: "".to_string(),
                    },
                )),
            }))
            .await
            .map_err(|e| TransactionError::new(String::from("Error sending commit message")))?;

        Ok(BranchStatus::PhaseTwoCommitted)
    }

    async fn branch_rollback(
        &self,
        global_session: &Self::GlobalSession,
        branch_session: &Self::BranchSession,
    ) -> Result<BranchStatus, TransactionError> {
        tracing::debug!(
            "TransactionCoordinatorOutbound branch_commit---{:?}",
            branch_session
        );

        let resource_id = branch_session.resource_id.clone().unwrap();
        let sender = {
            self.resources
                .read()
                .await
                .get(&resource_id)
                .ok_or_else(|| TransactionError::new(String::from("resource not found")))?
                .iter()
                .find(|rs| rs.resource.client_id == branch_session.client_id)
                .ok_or_else(|| TransactionError::new(String::from("client_id not found")))?
                .response_tx
                .clone()
        };
        sender
            .send(Ok(ResourceInstruction {
                instruction_id: 0,
                instruction: Some(resource_instruction::Instruction::Rollback(
                    BranchRollbackInstruction {
                        branch_type: branch_session.branch_type.into(),
                        xid: branch_session.xid.to_string(),
                        branch_id: branch_session.branch_id.into(),
                        resource_id: resource_id.0,
                        application_data: "".to_string(),
                    },
                )),
            }))
            .await
            .map_err(|e| TransactionError::new(String::from("Error sending commit message")))?;

        Ok(BranchStatus::PhaseTwoCommitted)
    }

    async fn branch_delete(
        &self,
        global_session: &Self::GlobalSession,
        branch_session: &Self::BranchSession,
    ) -> Result<BranchStatus, TransactionError> {
        self.branch_commit(global_session, branch_session).await
    }
}
