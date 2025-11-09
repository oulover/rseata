use crate::event::event_type::TransactionEventType;
use crate::types::Xid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub trait Event {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionEvent {
    pub event_id: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub timestamp: DateTime<Utc>,
    pub event_type: TransactionEventType,
    pub xid: Xid,
    pub application_id: String,
    pub transaction_name: String,
    pub metadata: serde_json::Value,
}
impl Event for TransactionEvent {}
