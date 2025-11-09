use crate::branch::BranchType;
use crate::coordinator::core_service::CoreService;
use crate::coordinator::{AbstractCore, Core};
use crate::session::branch_session::BranchSession;
use crate::session::global_session::GlobalSession;
use crate::transaction::transaction_manager::TransactionManager;
use std::sync::Arc;

pub trait CoreHolder: Core + CoreService + TransactionManager {
    type GlobalSession: GlobalSession + Send + Sync;
    type BranchSession: BranchSession + Send + Sync;
    fn get_core(
        &self,
        branch_type: BranchType,
    ) -> Arc<
        dyn AbstractCore<
                BranchSession = <Self as CoreHolder>::BranchSession,
                GlobalSession = <Self as CoreHolder>::GlobalSession,
            > + Send
            + Sync,
    >;
}
