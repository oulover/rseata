use crate::event::event::TransactionEvent;
use crate::event::event_error::EventError;
use crate::event::event_handler::EventHandler;
use crate::event::event_handler_chain::EventHandlerChain;
use crate::event::event_type::TransactionEventType;
use async_trait::async_trait;
use std::sync::Arc;

pub struct DefaultEventHandlerChain {
    handlers: Vec<Arc<dyn EventHandler<Event = TransactionEvent>>>,
}
impl DefaultEventHandlerChain {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }
}

#[async_trait]
impl EventHandlerChain for DefaultEventHandlerChain {
    type Event = TransactionEvent;

    async fn add_handler(&mut self, handler: Arc<dyn EventHandler<Event = Self::Event>>) {
        self.handlers.push(handler);
        // 按优先级排序
        self.handlers.sort_by_key(|h| h.priority());
    }

    async fn handle_event(&self, event: &TransactionEvent) -> Vec<Result<(), EventError>> {
        let mut results = Vec::new();
        for handler in &self.handlers {
            let interested_types = handler.interested_event_types();
            if interested_types.is_empty()
                || interested_types
                    .iter()
                    .any(|t| matches_event_type(t, &event.event_type))
            {
                let result = handler.handle_event(event).await;
                results.push(result);
            }
        }

        results
    }
}

fn matches_event_type(interested: &TransactionEventType, actual: &TransactionEventType) -> bool {
    std::mem::discriminant(interested) == std::mem::discriminant(actual)
}
