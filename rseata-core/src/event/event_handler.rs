use crate::event::event::Event;
use crate::event::event_error::EventError;
use crate::event::event_type::TransactionEventType;
use async_trait::async_trait;

#[async_trait]
pub trait EventHandler: Send + Sync {
    type Event: Event + Send + Sync;
    fn name(&self) -> &str;

    async fn handle_event(&self, event: &Self::Event) -> Result<(), EventError>;

    // 支持事件过滤
    fn interested_event_types(&self) -> &Vec<TransactionEventType>;

    // 事件处理优先级
    fn priority(&self) -> u16;
}
