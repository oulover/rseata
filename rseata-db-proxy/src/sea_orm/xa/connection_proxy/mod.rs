
mod impl_connection_trait;
mod impl_stream_trait;
mod impl_transaction_trait;

use crate::sea_orm::xa::transaction_proxy::XATransactionProxy;
use rseata_core::RSEATA_CLIENT_SESSION;
use rseata_core::types::Xid;
use sea_orm::ConnectionTrait;
use sea_orm::error::*;
use sea_orm::sqlx::types::uuid;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct XAId(pub String);

#[derive(Clone)]
pub struct XAConnectionProxy {
    pub url: String,
    pub xa_id: Arc<RwLock<Option<(XAId, Option<Xid>)>>>,
    pub is_xa_end: Arc<AtomicBool>,
    pub is_xa_prepare: Arc<AtomicBool>,
    pub sea_connection: sea_orm::DatabaseConnection,
}
impl XAConnectionProxy {
    pub async fn connect_mysql(url: &str) -> Result<Self, DbErr> {
        let t = sea_orm::Database::connect(url).await?;
        Ok(Self {
            url: url.to_string(),
            xa_id: Arc::new(RwLock::new(None)),
            is_xa_end: Arc::new(Default::default()),
            is_xa_prepare: Arc::new(Default::default()),
            sea_connection: t,
        })
    }
}

impl Deref for XAConnectionProxy {
    type Target = sea_orm::DatabaseConnection;

    fn deref(&self) -> &Self::Target {
        &self.sea_connection
    }
}
impl DerefMut for XAConnectionProxy {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sea_connection
    }
}

impl Debug for XAConnectionProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.sea_connection.fmt(f)
    }
}

pub fn get_url(url: &str) -> String {
    if url.contains("?") {
        let r = url.split("?");
        r.collect::<Vec<&str>>()[0].to_string()
    } else {
        url.to_string()
    }
}
