use crate::sea_orm::xa::transaction_proxy::{TransactionType, XATransactionProxy};
use sea_orm::{AccessMode, DbErr, IsolationLevel, RuntimeErr, TransactionTrait};
use std::fmt::{Debug, Display};
use std::pin::Pin;
use rseata_core::RSEATA_CLIENT_SESSION;
use rseata_tm::RSEATA_TM;

#[async_trait::async_trait]
impl TransactionTrait for XATransactionProxy {
    type Transaction = XATransactionProxy;

    async fn begin(&self) -> Result<Self::Transaction, DbErr> {
        self.xa_connection_proxy.begin().await
    }

    async fn begin_with_config(
        &self,
        isolation_level: Option<IsolationLevel>,
        access_mode: Option<AccessMode>,
    ) -> Result<Self::Transaction, DbErr> {
        self.xa_connection_proxy
            .begin_with_config(isolation_level, access_mode)
            .await
    }

    async fn transaction<F, T, E>(&self, callback: F) -> Result<T, sea_orm::TransactionError<E>>
    where
        F: for<'c> FnOnce(
                &'c Self::Transaction,
            ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'c>>
            + Send,
        T: Send,
        E: Display + Debug + Send,
    {
        self.xa_connection_proxy.transaction(callback).await
    }

    async fn transaction_with_config<F, T, E>(
        &self,
        callback: F,
        isolation_level: Option<IsolationLevel>,
        access_mode: Option<AccessMode>,
    ) -> Result<T, sea_orm::TransactionError<E>>
    where
        F: for<'c> FnOnce(
                &'c Self::Transaction,
            ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'c>>
            + Send,
        T: Send,
        E: Display + Debug + Send,
    {
        self.xa_connection_proxy
            .transaction_with_config(callback, isolation_level, access_mode)
            .await
    }
}
