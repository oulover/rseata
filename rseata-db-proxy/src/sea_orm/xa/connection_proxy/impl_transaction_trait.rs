use crate::sea_orm::xa::connection_proxy::XAConnectionProxy;
use crate::sea_orm::xa::transaction_proxy::XATransactionProxy;
use sea_orm::{
    AccessMode, DbErr, IsolationLevel, TransactionError,
    TransactionSession, TransactionTrait,
};
use std::env;
use std::fmt::{Debug, Display};
use std::pin::Pin;
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
        XATransactionProxy::new(self.clone(), isolation_level, access_mode).await
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
