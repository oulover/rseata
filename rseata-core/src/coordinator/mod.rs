use crate::branch::branch_manager_outbound::BranchManagerOutbound;
use crate::coordinator::transaction_coordinator_outbound::TransactionCoordinatorOutbound;
use crate::handle_branch_type::HandleBranchType;
use crate::resource::resource_registry::ResourceRegistry;
use crate::transaction::transaction_manager::TransactionManager;

pub mod core_holder;
pub mod transaction_coordinator_inbound;
pub mod transaction_coordinator_outbound;
pub mod core_service;

pub trait Coordinator: BranchManagerOutbound + ResourceRegistry + TransactionManager {}
pub trait Core:  BranchManagerOutbound {}
pub trait AbstractCore: TransactionCoordinatorOutbound +HandleBranchType + Core {}
