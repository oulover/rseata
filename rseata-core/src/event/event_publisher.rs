use crate::event::event::Event;
use async_trait::async_trait;

#[async_trait]
pub trait EventPublisher {
    type Event: Event + Send + Sync;
    async fn publish(&self, event: Self::Event);
}
