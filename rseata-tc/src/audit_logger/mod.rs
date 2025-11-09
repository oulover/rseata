use rseata_core::event::event::TransactionEvent;
use std::sync::Arc;
use tokio::sync::Mutex;

// pub trait AuditLogger {}

pub struct AuditLogger {
    pub log: Arc<Mutex<Vec<TransactionEvent>>>,
}
