pub mod impl_branch_manager_outbound;
pub mod impl_resource_registry;
mod impl_transaction_manager;

use crate::coordinator::default_core_holder::DefaultCoreHolder;
use crate::resource::TCResource;
use rseata_core::event::defaults::event_publisher::DefaultEventPublisher;
use rseata_core::types::ResourceId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use rseata_core::coordinator::Coordinator;

pub struct DefaultCoordinator {
    pub(crate) resources: Arc<RwLock<HashMap<ResourceId, Vec<TCResource>>>>,
    pub core: Arc<DefaultCoreHolder>,
    pub event_publisher: Arc<DefaultEventPublisher>,
}
impl DefaultCoordinator {
    pub fn new(event_publisher: Arc<DefaultEventPublisher>) -> Self {
        let resources: Arc<RwLock<HashMap<ResourceId, Vec<TCResource>>>> =
            Arc::new(Default::default());
        let core = DefaultCoreHolder::new_arc(resources.clone(), event_publisher.clone());
        Self {
            resources,
            core,
            event_publisher,
        }
    }
}

impl Coordinator for DefaultCoordinator {}
