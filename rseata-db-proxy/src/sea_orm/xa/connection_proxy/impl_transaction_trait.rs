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
use std::sync::atomic::Ordering;
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

            let xa_id = if should_begin_global_tx {
                // 首次
                self.xa_start( &None).await?
            } else {
                // 再次
                self.xa_start( &session.get_xid()).await?
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
                transaction_type: TransactionType::XA(xa_id),
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
     async fn execute_sql(
         &self,
        sql: &str,
    ) -> Result<(), DbErr> {
        let r =   self.one_connection.lock()
            .await
            .execute(sql)
            .await
            .map(|_| ())
            .map_err(|e| DbErr::Custom(e.to_string()));
        tracing::info!("**************------------------execute_sql executed  {sql}-------{:?}",r); // eab57cf7-1877-4e50-9e2f-fb7cfc76d6c7
        r
    }

    async fn xa_start(
        &self,
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
                    self.execute_sql( &sql).await?;
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
                self.execute_sql( &sql).await?;
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

    pub async fn xa_end(
        &self,
        xa_id: &XAId,
    ) -> Result<(), DbErr> {
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
        self.execute_sql( &sql).await?;

        self.is_xa_end.store(true, Ordering::Relaxed);
        Ok(())
    }
    pub async fn xa_prepare(
        &self,
        xa_id: &XAId,
    ) -> Result<(), DbErr> {
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
        self.execute_sql( &sql).await?;
        tracing::info!("xa_prepare ---- 33 XAConnectionProxy:xa_prepare------{sql}----");
        self.is_xa_prepare.store(true, Ordering::Relaxed);
        Ok(())
    }
    pub async fn xa_commit(
        &self,
        xid: &Xid,
    ) -> Result<(), DbErr> {
        let mut xa_id_lock = self.xa_id.write().await;
        if let Some((xa_id, xid_opt)) = xa_id_lock.as_ref() {
            if let Some(xid_opt) = xid_opt {
                if xid.eq(xid_opt) {
                    let sql = format!("XA COMMIT '{}'", xa_id.0);
                    self.execute_sql( &sql).await?;
                    *xa_id_lock = None;
                    self.is_xa_end.store(false, Ordering::Relaxed);
                    self.is_xa_prepare.store(false, Ordering::Relaxed);
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    pub async fn xa_rollback(
        &self,
        xid: &Xid,
    ) -> Result<(), DbErr> {
        let mut xa_id_lock = self.xa_id.write().await;
        if let Some((xa_id, xid_opt)) = xa_id_lock.as_ref() {
            if let Some(xid_opt) = xid_opt {
                if xid.eq(xid_opt) {
                    let sql = format!("XA ROLLBACK '{}'", xa_id.0);
                    self.execute_sql( &sql).await?;
                    *xa_id_lock = None;
                    self.is_xa_end.store(false, Ordering::Relaxed);
                    self.is_xa_prepare.store(false, Ordering::Relaxed);
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    pub async fn xa_rollback_xa_id(
        &self,
        xa_id: &XAId,
    ) -> Result<(), DbErr> {
        let mut xa_id_lock = self.xa_id.write().await;
        if let Some((xa_id_old, _)) = xa_id_lock.as_ref() {
            if xa_id_old.eq(xa_id) {
                let sql = format!("XA ROLLBACK '{}'", xa_id.0);
                self.execute_sql( &sql).await?;
                *xa_id_lock = None;
                self.is_xa_end.store(false, Ordering::Relaxed);
                self.is_xa_prepare.store(false, Ordering::Relaxed);
                return Ok(());
            }
        }
        Ok(())
    }
}

