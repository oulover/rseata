use crate::sea_orm::at::connection_proxy::ATConnectionProxy;
use crate::sea_orm::at::transaction_proxy::ATTransactionProxy;
use rseata_core::RSEATA_CLIENT_SESSION;
use rseata_core::transaction::transaction_manager::TransactionManager;
use rseata_tm::RSEATA_TM;
use sea_orm::{
    AccessMode, DbErr, IsolationLevel, RuntimeErr, TransactionError, TransactionSession,
    TransactionTrait,
};
use std::fmt::{Debug, Display};
use std::pin::Pin;

#[async_trait::async_trait]
impl TransactionTrait for ATConnectionProxy {
    type Transaction = ATTransactionProxy;

    async fn begin(&self) -> Result<Self::Transaction, DbErr> {
        self.begin_with_config(None, None).await
    }

    async fn begin_with_config(
        &self,
        isolation_level: Option<IsolationLevel>,
        access_mode: Option<AccessMode>,
    ) -> Result<Self::Transaction, DbErr> {
        println!("------begin_with_config----------------------");
        let result = self
            .sea_conn
            .begin_with_config(isolation_level, access_mode)
            .await;
        // 是否开启全局事务
        // 否：本地事务
        // 是：全局事务 TM.begin
        match result {
            Err(e) => Err(e),
            Ok(t) => {
                // TM获取全局锁
                let session = RSEATA_CLIENT_SESSION.try_with(|o| o.clone()).ok();
                if let Some(session) = session {
                    {
                        let should_begin_global_tx = { !session.is_global_tx_started() };

                        if should_begin_global_tx {
                            let xid = RSEATA_TM
                                .begin(
                                    "".to_string(),
                                    "".to_string(),
                                    session.transaction_name.clone(),
                                    100,
                                )
                                .await
                                .map_err(|e| DbErr::Conn(RuntimeErr::Internal(e.to_string())))?;

                            {
                                session
                                    .begin_global_transaction(xid)
                                    .map_err(|e| DbErr::Custom(e.to_string()))?;
                            }
                        }
                        session.init_branch().await;

                    }
                }
                Ok(ATTransactionProxy::new(self.clone(), t))
            }
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
