mod impl_connection_trait;
mod impl_stream_trait;
mod impl_transaction_session;
mod impl_transaction_trait;
mod impl_branch_transaction;

use crate::sea_orm::xa::connection_proxy::{XAConnectionProxy, XAId};
use rseata_core::RSEATA_CLIENT_SESSION;
use rseata_core::branch::BranchType;
use rseata_core::branch::branch_manager_outbound::BranchManagerOutbound;
use rseata_core::branch::branch_transaction::BranchTransactionRegistry;
use rseata_core::resource::Resource;
use rseata_core::types::Xid;
use rseata_rm::RSEATA_RM;
use sea_orm::sqlx::{Executor, Transaction};
use sea_orm::sqlx::pool::PoolConnection;
use sea_orm::sqlx::types::uuid;
use sea_orm::{ConnectionTrait, DatabaseTransaction, DbErr, TransactionTrait, sqlx};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::{Mutex, RwLock};

#[derive(Clone)]
pub enum TransactionType {
    Local( Arc<Mutex<Option<DatabaseTransaction>>> ),
    XA(XAId, Arc<Mutex<PoolConnection<sqlx::MySql>>>),
}

#[derive(Clone)]
pub struct XATransactionProxy {
    pub transaction_type: TransactionType,
    pub xa_connection_proxy: XAConnectionProxy,

}

impl std::fmt::Debug for XATransactionProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DatabaseTransaction")
    }
}

impl XATransactionProxy {
    pub async fn branch_register(&self, xa_id: &XAId) -> Result<(), DbErr> {
        let session = RSEATA_CLIENT_SESSION.try_get().ok();
        println!(
            "TransactionSession------branch_register----------------------{:?}",
            session
        );
        if let Some(session) = &session {
            // 注册 RM 分支事务
            println!("------------注册 RM 分支事务--ing---");
            let xid_guard = session.get_xid();
            if let Some(xid) = xid_guard {
                let lock_keys = session.get_branch_luck_keys().await.unwrap_or_default();
                let branch_id = RSEATA_RM
                    .branch_transaction_registry(
                        RSEATA_RM.resource_info.get_branch_type().await,
                        RSEATA_RM.resource_info.get_resource_id().await,
                        RSEATA_RM.resource_info.get_client_id().await,
                        xid,
                        "application_data".into(),
                        lock_keys,
                        Box::new(self.clone() ),
                    )
                    .await
                    .map_err(|e| DbErr::Custom(e.to_string()))?;
                println!("------------注册 RM 分支事务---完成{}", branch_id);
                session.set_branch_id(branch_id);
            }
        }
        Ok(())
    }

    pub async fn global_commit(local_commit_result: Result<(), DbErr>) -> Result<(), DbErr> {
        let session = RSEATA_CLIENT_SESSION.try_get().ok();
        println!("TransactionSession------commit----------------------");
        if let Some(session) = session {
            if session.is_global_tx_started() {
                if let Some(xid) = session.get_xid() {
                    let branch_status = match local_commit_result {
                        Ok(_) => rseata_core::branch::BranchStatus::PhaseOneDone,
                        Err(_) => rseata_core::branch::BranchStatus::PhaseOneFailed,
                    };
                    RSEATA_RM
                        .branch_report(
                            BranchType::AT,
                            xid,
                            session.get_branch_id(),
                            branch_status,
                            String::from(""),
                        )
                        .await
                        .map_err(|e| DbErr::Custom(e.to_string()))?;
                }
            }
        }
        local_commit_result.map(|_| ())
    }

    pub async fn global_rollback() -> Result<(), DbErr> {
        let session = RSEATA_CLIENT_SESSION.try_get().ok();
        println!(
            "TransactionSession------rollback----------------------{:?}",
            session
        );
        if let Some(session) = session {
            if session.is_global_tx_started() {
                if let Some(xid) = session.get_xid() {
                    let branch_status = rseata_core::branch::BranchStatus::PhaseOneFailed;
                    RSEATA_RM
                        .branch_report(
                            BranchType::AT,
                            xid,
                            session.get_branch_id(),
                            branch_status,
                            String::from(""),
                        )
                        .await
                        .map_err(|e| DbErr::Custom(e.to_string()))?;
                }
            }
        }
        Ok(())
    }

    pub async fn check_lock(&self) -> Result<bool, DbErr> {
        let session = RSEATA_CLIENT_SESSION.try_get().ok();
        if let Some(session) = &session {
            let xid_guard = session.get_xid();
            if let Some(xid) = xid_guard {
                let lock_keys = session.get_branch_luck_keys().await;
                if let Some(lock_keys) = lock_keys {
                    let locked = RSEATA_RM
                        .lock_query(
                            RSEATA_RM.resource_info.get_branch_type().await,
                            RSEATA_RM.resource_info.get_resource_id().await,
                            xid,
                            lock_keys,
                        )
                        .await
                        .map_err(|e| DbErr::Custom(e.to_string()))?;
                    return Ok(locked);
                }
            }
        }
        Ok(true)
    }
}

impl XATransactionProxy {
    pub(crate) async fn execute_sql(
        conn: Arc<Mutex<PoolConnection<sqlx::MySql>>>,
        sql: &str,
    ) -> Result<(), DbErr> {
      let r =   conn.lock()
            .await
            .execute(sql)
            .await
            .map(|_| ())
            .map_err(|e| DbErr::Custom(e.to_string()));
        tracing::info!("execute_sql executed  {sql}-------{:?}",r);
        r
    }

    pub async fn xa_end(
        &self,
        conn: Arc<Mutex<PoolConnection<sqlx::MySql>>>,
        xa_id: &XAId,
    ) -> Result<(), DbErr> {
        if self.xa_connection_proxy.is_xa_end.load(Ordering::Acquire) {
            let xa_id_lock = self.xa_connection_proxy.xa_id.read().await;
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
        XATransactionProxy::execute_sql(conn, &sql).await?;

        self.xa_connection_proxy.is_xa_end.store(true, Ordering::Relaxed);
        Ok(())
    }
    pub async fn xa_prepare(
        &self,
        conn: Arc<Mutex<PoolConnection<sqlx::MySql>>>,
        xa_id: &XAId,
    ) -> Result<(), DbErr> {
        if self.xa_connection_proxy.is_xa_prepare.load(Ordering::Acquire) {
            let xa_id_lock = self.xa_connection_proxy.xa_id.read().await;
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
        XATransactionProxy::execute_sql(conn, &sql).await?;
        tracing::info!("xa_prepare ---- 33 XAConnectionProxy:xa_prepare------{sql}----");
        self.xa_connection_proxy.is_xa_prepare.store(true, Ordering::Relaxed);
        Ok(())
    }
    pub async fn xa_commit(
        &self,
        conn: Arc<Mutex<PoolConnection<sqlx::MySql>>>,
        xid: &Xid,
    ) -> Result<(), DbErr> {
        let mut xa_id_lock = self.xa_connection_proxy.xa_id.write().await;
        if let Some((xa_id, xid_opt)) = xa_id_lock.as_ref() {
            if let Some(xid_opt) = xid_opt {
                if xid.eq(xid_opt) {
                    let sql = format!("XA COMMIT '{}'", xa_id.0);
                    XATransactionProxy::execute_sql(conn, &sql).await?;
                    *xa_id_lock = None;
                    self.xa_connection_proxy.is_xa_end.store(false, Ordering::Relaxed);
                    self.xa_connection_proxy.is_xa_prepare.store(false, Ordering::Relaxed);
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    pub async fn xa_rollback(
        &self,
        conn: Arc<Mutex<PoolConnection<sqlx::MySql>>>,
        xid: &Xid,
    ) -> Result<(), DbErr> {
        let mut xa_id_lock = self.xa_connection_proxy.xa_id.write().await;
        if let Some((xa_id, xid_opt)) = xa_id_lock.as_ref() {
            if let Some(xid_opt) = xid_opt {
                if xid.eq(xid_opt) {
                    let sql = format!("XA ROLLBACK '{}'", xa_id.0);
                    XATransactionProxy::execute_sql(conn, &sql).await?;
                    *xa_id_lock = None;
                    self.xa_connection_proxy.is_xa_end.store(false, Ordering::Relaxed);
                    self.xa_connection_proxy.is_xa_prepare.store(false, Ordering::Relaxed);
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    pub async fn xa_rollback_xa_id(
        &self,
        conn: Arc<Mutex<PoolConnection<sqlx::MySql>>>,
        xa_id: &XAId,
    ) -> Result<(), DbErr> {
        let mut xa_id_lock = self.xa_connection_proxy.xa_id.write().await;
        if let Some((xa_id_old, _)) = xa_id_lock.as_ref() {
            if xa_id_old.eq(xa_id) {
                let sql = format!("XA ROLLBACK '{}'", xa_id.0);
                XATransactionProxy::execute_sql(conn, &sql).await?;
                *xa_id_lock = None;
                self.xa_connection_proxy.is_xa_end.store(false, Ordering::Relaxed);
                self.xa_connection_proxy.is_xa_prepare.store(false, Ordering::Relaxed);
                return Ok(());
            }
        }
        Ok(())
    }
}
