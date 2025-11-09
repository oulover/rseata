use crate::sea_orm::at::transaction_proxy::ATTransactionProxy;
use sea_orm::{DbBackend, DbErr, Statement, StreamTrait, TransactionStream};
use std::pin::Pin;

impl StreamTrait for ATTransactionProxy {
    type Stream<'a> = TransactionStream<'a>;

    fn get_database_backend(&self) -> DbBackend {
        StreamTrait::get_database_backend(&self.at_connection_proxy)
    }
    fn stream_raw<'a>(
        &'a self,
        stmt: Statement,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Stream<'a>, DbErr>> + 'a + Send>> {
        self.sea_transaction.stream_raw(stmt)
    }
}
