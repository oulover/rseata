use tonic::async_trait;

#[async_trait]
pub trait ResourceRegistry {
    type Resource: crate::resource::Resource + Send + Sync;
    async fn register_resource(&self, resource: &Self::Resource);
    async fn unregister_resource(&mut self, resource: &Self::Resource);
}
