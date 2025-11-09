use crate::branch::branch_manager_outbound::BranchManagerOutbound;
use crate::transaction::transaction_manager::TransactionManager;

pub trait TransactionCoordinatorInbound: BranchManagerOutbound + TransactionManager {}
