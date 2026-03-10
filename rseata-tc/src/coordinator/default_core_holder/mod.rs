pub mod impl_branch_manager_outbound;
pub mod impl_core_holder;
pub mod impl_transaction_manager;

use crate::coordinator::core::at_core::ATCore;
use crate::coordinator::core::xa_core::XACore;
use crate::resource::TCResource;
use rseata_core::event::defaults::event_publisher::DefaultEventPublisher;
use rseata_core::lock::defaults::default_lock_manager::DefaultLockManager;
use rseata_core::lock::defaults::default_locker::MemoryLocker;
use rseata_core::session::defaults::default_session_manager::DefaultSessionManager;
use rseata_core::store::memery_transaction_store_manager::MemeryTransactionStoreManager;
use rseata_core::types::ResourceId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use rseata_core::coordinator::{AbstractCore, Core};
use rseata_core::coordinator::core_holder::CoreHolder;
use rseata_core::coordinator::core_service::CoreService;

pub struct DefaultCoreHolder {
    pub(crate) session_manager: Arc<DefaultSessionManager>,
    pub(crate) resources: Arc<RwLock<HashMap<ResourceId, Vec<TCResource>>>>,
    pub(crate) at_core: Arc<
        dyn AbstractCore<
                BranchSession = <Self as CoreHolder>::BranchSession,
                GlobalSession = <Self as CoreHolder>::GlobalSession,
            > + Send
            + Sync,
    >,
    pub(crate) xa_core: Arc<
        dyn AbstractCore<
                BranchSession = <Self as CoreHolder>::BranchSession,
                GlobalSession = <Self as CoreHolder>::GlobalSession,
            > + Send
            + Sync,
    >,
    pub(crate) event_publisher: Arc<DefaultEventPublisher>,
}
impl DefaultCoreHolder {
    pub(crate) fn new_arc(
        resources: Arc<RwLock<HashMap<ResourceId, Vec<TCResource>>>>,
        event_publisher: Arc<DefaultEventPublisher>,
    ) -> Arc<Self> {
        let session_manager = Arc::new(DefaultSessionManager::new(
            String::from("DefaultSessionManager"),
            Box::new(MemeryTransactionStoreManager::default()),
        ));

        let resources2 = resources.clone();
        Arc::new(Self {
            session_manager: session_manager.clone(),
            resources: resources.clone(),
            at_core: Arc::new(ATCore {
                session_manager: session_manager.clone(),
                lock_manager: Arc::new(DefaultLockManager::new(Arc::new(MemoryLocker::default()))),
                resources,
                event_publisher: event_publisher.clone(),
            }),
            xa_core: Arc::new(XACore {
                session_manager: session_manager.clone(),
                lock_manager: Arc::new(DefaultLockManager::new(Arc::new(MemoryLocker::default()))),
                resources: resources2,
                event_publisher: event_publisher.clone(),
            }),
            event_publisher,
        })
    }
}

impl Core for DefaultCoreHolder {}

impl CoreService for DefaultCoreHolder {}

#[cfg(test)]
mod tests {
    use super::*;
    use rseata_core::branch::BranchType;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use std::collections::HashMap;

    #[test]
    fn test_get_core_by_branch_type() {
        let resources = Arc::new(RwLock::new(HashMap::new()));
        let event_publisher = Arc::new(DefaultEventPublisher::default());
        let holder = DefaultCoreHolder::new_arc(resources, event_publisher);

        let at_core = holder.get_core(BranchType::AT);
        let xa_core = holder.get_core(BranchType::XA);
        // They should be different Arc pointers (different cores)
        assert!(!Arc::ptr_eq(&at_core, &xa_core));
        // Ensure they return correct branch types via HandleBranchType trait
        assert_eq!(at_core.handle_branch_type(), BranchType::AT);
        assert_eq!(xa_core.handle_branch_type(), BranchType::XA);
    }
}
