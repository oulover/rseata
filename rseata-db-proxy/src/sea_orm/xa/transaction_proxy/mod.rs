mod impl_branch_transaction;
mod impl_connection_trait;
mod impl_stream_trait;
mod impl_transaction_session;
mod impl_transaction_trait;

use crate::sea_orm::xa::connection_proxy::{XAConnectionProxy, XAId};
use rseata_core::RSEATA_CLIENT_SESSION;
use rseata_core::branch::BranchType;
use rseata_core::branch::branch_manager_outbound::BranchManagerOutbound;
use rseata_core::branch::branch_transaction::BranchTransactionRegistry;
use rseata_core::resource::Resource;
use rseata_core::transaction::transaction_manager::TransactionManager;
use rseata_core::types::Xid;
use rseata_rm::RSEATA_RM;
use rseata_tm::RSEATA_TM;
use sea_orm::sqlx::pool::PoolConnection;
use sea_orm::sqlx::types::uuid;
use sea_orm::sqlx::{Executor, MySqlConnection, Transaction};
use sea_orm::{
    AccessMode, ConnectionTrait, DatabaseTransaction, DbErr, IsolationLevel, RuntimeErr,
    TransactionTrait, sqlx,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::{Mutex, RwLock};

#[derive(Clone)]
pub struct XATransaction {
    pub xa_id: XAId,
    pub xid: Xid,
    pub connection: Arc<Mutex<MySqlConnection>>,
}

impl XATransaction {
    async fn execute_sql(&self, sql: &str) -> Result<(), DbErr> {
        let r = self
            .connection
            .lock()
            .await
            .execute(sql)
            .await
            .map(|_| ())
            .map_err(|e| DbErr::Custom(e.to_string()));
        tracing::info!("XATransaction-----execute_sql executed {sql}---{:?}", r);
        r
    }

    async fn xa_start(conn: &mut MySqlConnection) -> Result<(XAId), DbErr> {
        let xa_id = uuid::Uuid::new_v4().to_string();
        let sql = format!("XA START '{}'", xa_id);
        conn.execute(sql.as_str())
            .await
            .map(|_| XAId(xa_id))
            .map_err(|e| DbErr::Custom(e.to_string()))
    }

    pub async fn xa_end(&self) -> Result<(), DbErr> {
        let sql = format!("XA END '{}'", self.xa_id.0);
        self.execute_sql(&sql).await
    }
    pub async fn xa_prepare(&self) -> Result<(), DbErr> {
        let sql = format!("XA PREPARE '{}'", self.xa_id.0);
        self.execute_sql(&sql).await
    }
    pub async fn xa_commit(&self) -> Result<(), DbErr> {
        let sql = format!("XA COMMIT '{}'", self.xa_id.0);
        self.execute_sql(&sql).await
    }

    pub async fn xa_rollback(&self) -> Result<(), DbErr> {
        let sql = format!("XA ROLLBACK '{}'", self.xa_id.0);
        self.execute_sql(&sql).await
    }
}

#[derive(Clone)]
pub enum TransactionType {
    Local(Arc<Mutex<Option<DatabaseTransaction>>>),
    XA(XATransaction),
}

#[derive(Clone)]
pub struct XATransactionProxy {
    transaction_type: TransactionType,
    xa_connection_proxy: XAConnectionProxy,
}
impl XATransactionProxy {
    pub(crate) async fn new(
        xa_connection_proxy: XAConnectionProxy,
        isolation_level: Option<IsolationLevel>,
        access_mode: Option<AccessMode>,
    ) -> Result<XATransactionProxy, DbErr> {
        let session = RSEATA_CLIENT_SESSION.try_get().ok();
        if let Some(session) = session {
            let mut conn = xa_connection_proxy
                .sea_connection
                .get_mysql_connection_pool()
                .acquire()
                .await
                .map_err(|err| DbErr::Custom(err.to_string()))?
                .detach();
            let should_begin_global_tx = { !session.is_global_tx_started() };

            let xa_id = XATransaction::xa_start(&mut conn).await?;

            tracing::info!("begin_with_config--{should_begin_global_tx}-{}", xa_id.0);
            let mut xid_init = session.get_xid();

            if should_begin_global_tx {
                let xid = RSEATA_TM
                    .begin(
                        RSEATA_TM.application_id.to_string(),
                        RSEATA_TM.transaction_service_group.to_string(),
                        session.transaction_name.clone(),
                        100,
                    )
                    .await
                    .map_err(|e| DbErr::Conn(RuntimeErr::Internal(e.to_string())))?;

                {
                    session
                        .begin_global_transaction(xid.clone())
                        .map_err(|e| DbErr::Custom(e.to_string()))?;
                }
                xid_init = Some(xid);
            }

            session.init_branch().await;

            Ok(XATransactionProxy {
                transaction_type: TransactionType::XA(XATransaction {
                    xa_id,
                    xid: xid_init.ok_or(DbErr::Custom("XID initialization failed".to_string()))?,
                    connection: Arc::new(Mutex::new(conn)),
                }),
                xa_connection_proxy: xa_connection_proxy.clone(),
            })
        } else {
            let local = xa_connection_proxy
                .sea_connection
                .begin_with_config(isolation_level, access_mode)
                .await?;
            Ok(XATransactionProxy {
                transaction_type: TransactionType::Local(Arc::new(Mutex::new(Some(local)))),
                xa_connection_proxy: xa_connection_proxy.clone(),
            })
        }
    }
}

impl std::fmt::Debug for XATransactionProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DatabaseTransaction")
    }
}

impl XATransactionProxy {
    pub async fn branch_register(&self) -> Result<(), DbErr> {
        let session = RSEATA_CLIENT_SESSION.try_get().ok();
        if let Some(session) = &session {
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
                        Box::new(self.clone()),
                    )
                    .await
                    .map_err(|e| DbErr::Custom(e.to_string()))?;
                session.set_branch_id(branch_id);
            }
        }
        Ok(())
    }

    pub async fn report_local_commit(local_commit_result: Result<(), DbErr>) -> Result<(), DbErr> {
        let session = RSEATA_CLIENT_SESSION.try_get().ok();
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

    pub async fn report_local_rollback() -> Result<(), DbErr> {
        let session = RSEATA_CLIENT_SESSION.try_get().ok();
       
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
        Ok(true)
    }
}
