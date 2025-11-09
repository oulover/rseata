use crate::sea_orm::connection_proxy::ConnectionProxy;
use sea_orm::{DbBackend, DbErr, QueryStream, Statement, StreamTrait};
use std::pin::Pin;

#[async_trait::async_trait]
impl StreamTrait for ConnectionProxy {
    type Stream<'a> = QueryStream;

    fn get_database_backend(&self) -> DbBackend {
        self.inner.get_database_backend()
    }

    fn stream_raw<'a>(
        &'a self,
        stmt: Statement,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Stream<'a>, DbErr>> + 'a + Send>> {
        self.inner.stream_raw(stmt)
    }
}
