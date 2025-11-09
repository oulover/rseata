use crate::audit_logger::AuditLogger;
use async_trait::async_trait;
use rseata_core::event::event::TransactionEvent;
use rseata_core::event::event_error::EventError;
use rseata_core::event::event_handler::EventHandler;
use rseata_core::event::event_type::TransactionEventType;
use std::sync::Arc;

pub struct AuditLogEventHandler {
    name: String,
    logger: Arc<AuditLogger>,
    interested_event_types: Vec<TransactionEventType>,
}
impl AuditLogEventHandler {
    pub fn new(logger: Arc<AuditLogger>) -> Self {
        Self {
            name: "".to_string(),
            logger,
            interested_event_types: Vec::new(),
        }
    }
}

#[async_trait]
impl EventHandler for AuditLogEventHandler {
    type Event = TransactionEvent;

    fn name(&self) -> &str {
        &self.name
    }

    async fn handle_event(&self, event: &Self::Event) -> Result<(), EventError> {
        let mut r = self.logger.log.lock().await;
        if r.len() > 300 {
            r.clear();
        }
        r.push(event.clone());
        Ok(())
    }

    fn interested_event_types(&self) -> &Vec<TransactionEventType> {
        &self.interested_event_types
    }

    fn priority(&self) -> u16 {
        1
    }
}

// fn name(&self) -> &str {}
//
// async fn handle_event(&self, event: &TransactionEvent) -> Result<(), SeataError> {
//     let log_entry = AuditLogEntry {
//         timestamp: event.timestamp,
//         event_id: event.event_id.clone(),
//         xid: event.xid.clone(),
//         event_type: format!("{:?}", event.event_type),
//         application_id: event.application_id.clone(),
//         metadata: event.metadata.clone(),
//     };
//
//     self.logger.log(log_entry).await?;
//     Ok(())
// }
