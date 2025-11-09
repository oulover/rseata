use crate::sea_orm::at::transaction_proxy::ATTransactionProxy;
use sea_orm::{AccessMode, DbErr, IsolationLevel, TransactionTrait};
use std::fmt::{Debug, Display};
use std::pin::Pin;

#[async_trait::async_trait]
impl TransactionTrait for ATTransactionProxy {
    type Transaction = ATTransactionProxy;

    async fn begin(&self) -> Result<Self::Transaction, DbErr> {
        println!("ATTransactionProxy------------begin");
        self.at_connection_proxy.begin().await
    }

    async fn begin_with_config(
        &self,
        isolation_level: Option<IsolationLevel>,
        access_mode: Option<AccessMode>,
    ) -> Result<Self::Transaction, DbErr> {
        println!("ATTransactionProxy------------begin_with_config");
        self.at_connection_proxy
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
        println!("ATTransactionProxy------------transaction");
        self.at_connection_proxy.transaction(callback).await
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
        println!("ATTransactionProxy------------transaction_with_config");
        self.at_connection_proxy
            .transaction_with_config(callback, isolation_level, access_mode)
            .await
    }
}
