use crate::event::event::TransactionEvent;
use crate::event::event_error::EventError;
use crate::event::event_handler::EventHandler;
use crate::event::event_type::TransactionEventType;
use async_trait::async_trait;

pub struct DefaultPrintEventHandler {
    name: String,
    interested_event_types: Vec<TransactionEventType>,
}
#[async_trait]
impl EventHandler for DefaultPrintEventHandler {
    type Event = TransactionEvent;

    fn name(&self) -> &str {
        &self.name
    }

    async fn handle_event(&self, event: &Self::Event) -> Result<(), EventError> {
        println!("{} is handling event {:?}", self.name, event);
        Ok(())
    }

    fn interested_event_types(&self) -> &Vec<TransactionEventType> {
        &self.interested_event_types
    }

    fn priority(&self) -> u16 {
        1
    }
}
