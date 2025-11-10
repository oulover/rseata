mod impl_branch_transaction;
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
    xa_id: Arc<RwLock<Option<(XAId, Option<Xid>)>>>,
    is_xa_end: Arc<AtomicBool>,
    is_xa_prepare: Arc<AtomicBool>,
    pub sea_connection: sea_orm::DatabaseConnection,
}
impl XAConnectionProxy {
    pub async fn connect_mysql(url: &str) -> Result<Self, DbErr> {
        let t = sea_orm::Database::connect(url).await?;
        Ok(Self {
            url: url.to_string(),
            xa_id: Arc::new(RwLock::new(None)),
            is_xa_end: Arc::new(AtomicBool::new(false)),
            is_xa_prepare: Arc::new(AtomicBool::new(false)),
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

impl XAConnectionProxy {
    async fn xa_start(&self, xid: &Option<Xid>) -> Result<XAId, DbErr> {
        let mut xa_id_lock = self.xa_id.write().await;
        tracing::info!(
            "XAConnectionProxy:xa_prepare------{:?}----{:?}",
            xid,
            xa_id_lock
        );
        if let Some(xid) = xid {
            // 再次
            if let Some((xa_id_lock, xid_lock)) = xa_id_lock.as_ref() {
                if let Some(xid_lock) = xid_lock {
                    if xid.eq(xid_lock) {
                        Ok(xa_id_lock.clone())
                    } else {
                        Err(DbErr::Custom(format!(
                            "XAConnectionProxy ax started xa_id: {},but not your xid :{}",
                            xa_id_lock.0, xid
                        )))
                    }
                } else {
                    Err(DbErr::Custom(format!(
                        "XAConnectionProxy ax started xa_id: {},but xid not set",
                        xa_id_lock.0
                    )))
                }
            } else {
                // 当作首次
                if xa_id_lock.is_none() {
                    let xa_id = uuid::Uuid::new_v4().to_string();
                    let sql = format!("XA START '{xa_id}'");
                    self.execute_unprepared(&sql).await?;
                    *xa_id_lock = Some((XAId(xa_id.clone()), None));
                    Ok(XAId(xa_id))
                } else {
                    Err(DbErr::Custom(
                        "XAConnectionProxy ax started xa_id: ,but not contains you".to_string(),
                    ))
                }
            }
        } else {
            // 首次

            if xa_id_lock.is_none() {
                let xa_id = uuid::Uuid::new_v4().to_string();
                let sql = format!("XA START '{xa_id}'");
                self.execute_unprepared(&sql).await?;
                *xa_id_lock = Some((XAId(xa_id.clone()), None));
                Ok(XAId(xa_id))
            } else {
                tracing::info!("XAConnectionProxy:xa_start {:?}", xa_id_lock);
                Err(DbErr::Custom(
                    "XAConnectionProxy ax started xa_id: ,but not contains you".to_string(),
                ))
            }
        }
    }
    pub async fn xa_end(&self, xa_id: &XAId) -> Result<(), DbErr> {
        // let xa_id_lock = self.xa_id.read().await;
        // tracing::info!("xa_end ---- 11 XAConnectionProxy:xa_end------{:?}----{:?}",xa_id,xa_id_lock);
        // if let Some((xa_id_old, _)) = xa_id_lock.as_ref() {
        //     if xa_id.eq(xa_id_old) {
        //         tracing::info!("xa_end ---- 22 XAConnectionProxy:xa_end------{:?}----{:?}",xa_id,xa_id_lock);
        //         return Ok(());
        //     }
        // }

        if self.is_xa_end.load(Ordering::Acquire) {
            let xa_id_lock = self.xa_id.read().await;
            tracing::info!(
                "xa_end ---- 11 XAConnectionProxy:xa_end------{:?}----{:?}",
                xa_id,
                xa_id_lock
            );
            if let Some((xa_id_old, _)) = xa_id_lock.as_ref() {
                if xa_id.eq(xa_id_old) {
                    tracing::info!(
                        "xa_end ---- 22 XAConnectionProxy:xa_end------{:?}----{:?}",
                        xa_id,
                        xa_id_lock
                    );
                    return Ok(());
                }
            }
        }

        let sql = format!("XA END '{}'", xa_id.0);
        tracing::info!("xa_end ---- 33 XAConnectionProxy:xa_end------{sql}----");
        self.execute_unprepared(&sql).await?;
        self.is_xa_end.store(true, Ordering::Relaxed);
        Ok(())
    }
    pub async fn xa_prepare(&self, xa_id: &XAId) -> Result<(), DbErr> {
        if self.is_xa_prepare.load(Ordering::Acquire) {
            let xa_id_lock = self.xa_id.read().await;
            tracing::info!(
                "xa_prepare ---- 11 XAConnectionProxy:xa_prepare------{:?}----{:?}",
                xa_id,
                xa_id_lock
            );
            if let Some((xa_id_old, _)) = xa_id_lock.as_ref() {
                if xa_id.eq(xa_id_old) {
                    tracing::info!(
                        "xa_prepare ---- 22 XAConnectionProxy:xa_prepare------{:?}----{:?}",
                        xa_id,
                        xa_id_lock
                    );
                    return Ok(());
                }
            }
        }

        let sql = format!("XA PREPARE '{}'", xa_id.0);
        tracing::info!("xa_prepare ---- 33 XAConnectionProxy:xa_prepare------{sql}----");
        self.is_xa_prepare.store(true, Ordering::Relaxed);
        Ok(())
    }
    pub async fn xa_commit(&self, xid: &Xid) -> Result<(), DbErr> {
        let mut xa_id_lock = self.xa_id.write().await;
        if let Some((xa_id, xid_opt)) = xa_id_lock.as_ref() {
            if let Some(xid_opt) = xid_opt {
                if xid.eq(xid_opt) {
                    let sql = format!("XA COMMIT '{}'", xa_id.0);
                    self.execute_unprepared(&sql).await?;
                    *xa_id_lock = None;
                    self.is_xa_end.store(false, Ordering::Relaxed);
                    self.is_xa_prepare.store(false, Ordering::Relaxed);
                    return Ok(());
                }
            }
        }
        Ok(())
    }


    pub async fn xa_rollback(&self, xid: &Xid) -> Result<(), DbErr> {
        let mut xa_id_lock = self.xa_id.write().await;
        if let Some((xa_id, xid_opt)) = xa_id_lock.as_ref() {
            if let Some(xid_opt) = xid_opt {
                if xid.eq(xid_opt) {

                    let sql = format!("XA ROLLBACK '{}'", xa_id.0);
                    self.execute_unprepared(&sql).await?;
                    *xa_id_lock = None;
                    self.is_xa_end.store(false, Ordering::Relaxed);
                    self.is_xa_prepare.store(false, Ordering::Relaxed);
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    pub async fn xa_rollback_xa_id(&self, xa_id: &XAId) -> Result<(), DbErr> {
        let mut xa_id_lock = self.xa_id.write().await;
        if let Some((xa_id_old, _)) = xa_id_lock.as_ref() {
            if xa_id_old.eq(xa_id) {
                let sql = format!("XA ROLLBACK '{}'", xa_id.0);
                self.execute_unprepared(&sql).await?;
                *xa_id_lock = None;
                self.is_xa_end.store(false, Ordering::Relaxed);
                self.is_xa_prepare.store(false, Ordering::Relaxed);
                return Ok(());
            }
        }
        Ok(())
    }
}
