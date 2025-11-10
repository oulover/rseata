use crate::sea_orm::xa::connection_proxy::{XAConnectionProxy, XAId};
use crate::sea_orm::xa::transaction_proxy::{TransactionType, XATransactionProxy};
use rseata_core::RSEATA_CLIENT_SESSION;
use rseata_core::transaction::transaction_manager::TransactionManager;
use rseata_core::types::Xid;
use rseata_tm::RSEATA_TM;
use sea_orm::sqlx::pool::PoolConnection;
use sea_orm::sqlx::types::uuid;
use sea_orm::sqlx::{Acquire, Executor};
use sea_orm::{
    AccessMode, ConnectionTrait, DbErr, IsolationLevel, RuntimeErr, TransactionError,
    TransactionSession, TransactionTrait, sqlx,
};
use std::env;
use std::fmt::{Debug, Display};
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;

fn get_app_id() -> String {
    env::var("APP_ID").expect("APP_ID not set")
}

#[async_trait::async_trait]
impl TransactionTrait for XAConnectionProxy {
    type Transaction = XATransactionProxy;

    async fn begin(&self) -> Result<Self::Transaction, DbErr> {
        self.begin_with_config(None, None).await
    }

    async fn begin_with_config(
        &self,
        isolation_level: Option<IsolationLevel>,
        access_mode: Option<AccessMode>,
    ) -> Result<Self::Transaction, DbErr> {
        let session = RSEATA_CLIENT_SESSION.try_get().ok();
        if let Some(session) = session {
            let should_begin_global_tx = { !session.is_global_tx_started() };
            let conn = self
                .sea_connection
                .get_mysql_connection_pool()
                .acquire()
                .await
                .map_err(|err| DbErr::Custom(err.to_string()))?;

            let conn = Arc::new(Mutex::new(conn));

            let xa_id = if should_begin_global_tx {
                // 首次
                self.xa_start(conn.clone(), &None).await?
            } else {
                // 再次
                self.xa_start(conn.clone(), &session.get_xid()).await?
            };
            tracing::info!("begin_with_config--{should_begin_global_tx}-{}", xa_id.0);

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
                {
                    let mut xa_id_lock = self.xa_id.write().await;
                    if let Some((xa_id, _)) = xa_id_lock.take() {
                        *xa_id_lock = Some((xa_id, Some(xid)));
                    }
                }
            }

            session.init_branch().await;

            Ok(XATransactionProxy {
                transaction_type: TransactionType::XA(XAId(uuid::Uuid::new_v4().to_string()), conn),
                xa_connection_proxy: self.clone(),
            })
        } else {
            let local = self
                .sea_connection
                .begin_with_config(isolation_level, access_mode)
                .await?;
            Ok(XATransactionProxy {
                transaction_type: TransactionType::Local(Arc::new(Mutex::new(Some(
                    local,
                )))),
                xa_connection_proxy: self.clone(),
            })
        }
    }

    async fn transaction<F, T, E>(&self, callback: F) -> Result<T, TransactionError<E>>
    where
        F: for<'c> FnOnce(
                &'c Self::Transaction,
            ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'c>>
            + Send,
        T: Send,
        E: Display + Debug + Send,
    {
        self.transaction_with_config(callback, None, None).await
    }

    async fn transaction_with_config<F, T, E>(
        &self,
        callback: F,
        isolation_level: Option<IsolationLevel>,
        access_mode: Option<AccessMode>,
    ) -> Result<T, TransactionError<E>>
    where
        F: for<'c> FnOnce(
                &'c Self::Transaction,
            ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'c>>
            + Send,
        T: Send,
        E: Display + Debug + Send,
    {
        let txn = self.begin_with_config(isolation_level, access_mode).await?;
        let res = callback(&txn).await.map_err(TransactionError::Transaction);
        if res.is_ok() {
            txn.commit().await.map_err(TransactionError::Connection)?;
        } else {
            txn.rollback().await.map_err(TransactionError::Connection)?;
        }
        res
    }
}

impl XAConnectionProxy {
    async fn xa_start(
        &self,
        conn: Arc<Mutex<PoolConnection<sqlx::MySql>>>,
        xid: &Option<Xid>,
    ) -> Result<XAId, DbErr> {
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
                    XATransactionProxy::execute_sql(conn, &sql).await?;
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
                XATransactionProxy::execute_sql(conn, &sql).await?;
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
}
