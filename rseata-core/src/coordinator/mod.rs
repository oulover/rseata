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





// BranchSessionService queryByXid deleteBranchSession forceDeleteBranchSession : SessionLifecycle.changeBranchStatus
// GlobalLockService:queryGlobalByXid deleteLock query check
// AbstractGlobalService  deleteGlobalSession: SessionLifecycle.changeGlobalStatus(GlobalStatus.Deleting);
// session :  SessionLifecycle



// DefaultCore 更具branch type 发送 给 不同的 Core 处理
// CoreService 提供执行接口 给web 调用
