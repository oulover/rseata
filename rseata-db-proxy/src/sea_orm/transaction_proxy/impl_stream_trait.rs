use crate::sea_orm::transaction_proxy::TransactionProxy;
use sea_orm::{DbBackend, DbErr, Statement, StreamTrait, TransactionStream};
use std::pin::Pin;

impl StreamTrait for TransactionProxy {
    type Stream<'a> = TransactionStream<'a>;

    fn get_database_backend(&self) -> DbBackend {
        StreamTrait::get_database_backend(&self.inner)
    }
    fn stream_raw<'a>(
        &'a self,
        stmt: Statement,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Stream<'a>, DbErr>> + 'a + Send>> {
        self.inner.stream_raw(stmt)
    }
}
