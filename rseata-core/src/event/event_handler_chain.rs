use crate::event::event::{Event, TransactionEvent};
use crate::event::event_error::EventError;
use crate::event::event_handler::EventHandler;
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait EventHandlerChain {
    type Event: Event;
    async fn add_handler(&mut self, handler: Arc<dyn EventHandler<Event = Self::Event>>);

    async fn handle_event(&self, event: &TransactionEvent) -> Vec<Result<(), EventError>>;
}
