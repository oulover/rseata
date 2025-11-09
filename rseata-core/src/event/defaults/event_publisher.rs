use crate::event::defaults::default_event_handler_chain::DefaultEventHandlerChain;
use crate::event::event::TransactionEvent;
use crate::event::event_handler_chain::EventHandlerChain;
use crate::event::event_publisher::EventPublisher;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct DefaultEventPublisher {
    event_tx: mpsc::UnboundedSender<TransactionEvent>,
    handler_chain: Arc<DefaultEventHandlerChain>,
}

impl DefaultEventPublisher {
    pub fn new(handler_chain: Arc<DefaultEventHandlerChain>) -> Self {
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();
        let publisher = Self {
            event_tx,
            handler_chain: handler_chain.clone(),
        };

        // 启动事件处理循环
        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                let chain = handler_chain.clone();
                // 异步处理事件，不阻塞发布者
                tokio::spawn(async move {
                    let _ = chain.handle_event(&event).await;
                });
            }
        });
        publisher
    }
}
#[async_trait]
impl EventPublisher for DefaultEventPublisher {
    type Event = TransactionEvent;

    async fn publish(&self, event: Self::Event) {
        let _ = self.event_tx.send(event);
    }
}
