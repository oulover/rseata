use crate::coordinator::default_coordinator::DefaultCoordinator;
use crate::resource::TCResource;
use async_trait::async_trait;
use rseata_core::event::event::TransactionEvent;
use rseata_core::event::event_publisher::EventPublisher;
use rseata_core::event::event_type::TransactionEventType;
use rseata_core::resource::resource_registry::ResourceRegistry;
use rseata_core::types::Xid;
use uuid::Uuid;

#[async_trait]
impl ResourceRegistry for DefaultCoordinator {
    type Resource = TCResource;

    async fn register_resource(&self, resource: &Self::Resource) {
        tracing::info!("Registering resource {:?}", resource.resource);
        self.event_publisher
            .publish(TransactionEvent {
                event_id: Uuid::new_v4().to_string(),
                timestamp: Default::default(),
                event_type: TransactionEventType::ResourceRegistered {
                    resource_id: resource.resource.resource_id.clone(),
                    branch_type: resource.resource.branch_type,
                },
                xid: Xid(String::new()),
                application_id: "".to_string(),
                transaction_name: "".to_string(),
                metadata: Default::default(),
            })
            .await;
        self.resources
            .write()
            .await
            .entry(resource.resource.resource_id.clone())
            .or_default()
            .push(resource.clone());
    }

    async fn unregister_resource(&mut self, resource: &Self::Resource) {}
}
