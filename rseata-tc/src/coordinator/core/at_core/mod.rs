pub mod impl_branch_manager_outbound;
pub mod impl_transaction_coordinator_outbound;

use crate::resource::TCResource;
use rseata_core::branch::BranchType;
use rseata_core::coordinator::{AbstractCore, Core};
use rseata_core::event::defaults::event_publisher::DefaultEventPublisher;
use rseata_core::handle_branch_type::HandleBranchType;
use rseata_core::lock::defaults::default_lock_manager::DefaultLockManager;
use rseata_core::lock::defaults::default_locker::MemoryLocker;
use rseata_core::session::defaults::default_session_manager::DefaultSessionManager;
use rseata_core::types::ResourceId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ATCore {
    pub(crate) session_manager: Arc<DefaultSessionManager>,
    pub(crate) lock_manager: Arc<DefaultLockManager<MemoryLocker>>,
    pub(crate) resources: Arc<RwLock<HashMap<ResourceId, Vec<TCResource>>>>,
    pub(crate) event_publisher: Arc<DefaultEventPublisher>,
}

impl HandleBranchType for ATCore {
    fn handle_branch_type(&self) -> BranchType {
        BranchType::AT
    }
}

impl Core for ATCore {}

impl AbstractCore for ATCore {}
