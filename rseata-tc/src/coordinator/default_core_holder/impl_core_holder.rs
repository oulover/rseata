use std::sync::Arc;
use rseata_core::branch::BranchType;
use rseata_core::coordinator::AbstractCore;
use rseata_core::coordinator::core_holder::CoreHolder;
use rseata_core::session::defaults::default_branch_session::DefaultBranchSession;
use rseata_core::session::defaults::default_global_session::DefaultGlobalSession;
use crate::coordinator::default_core_holder::DefaultCoreHolder;

impl CoreHolder for DefaultCoreHolder {
    type GlobalSession = DefaultGlobalSession;
    type BranchSession = DefaultBranchSession;

    fn get_core(
        &self,
        branch_type: BranchType,
    ) -> Arc<
        dyn AbstractCore<
            BranchSession = <Self as CoreHolder>::BranchSession,
            GlobalSession = <Self as CoreHolder>::GlobalSession,
        > + Send
        + Sync,
    > {
        match branch_type {
            _ => {}
        }
        self.at_core.clone()
    }
}
