use crate::sea_orm::xa::connection_proxy::XAConnectionProxy;
use sea_orm::{DbBackend, DbErr, QueryStream, Statement, StreamTrait};
use std::pin::Pin;

#[async_trait::async_trait]
impl StreamTrait for XAConnectionProxy {
    type Stream<'a> = QueryStream;

    fn get_database_backend(&self) -> DbBackend {
        self.sea_connection.get_database_backend()
    }

    fn stream_raw<'a>(
        &'a self,
        stmt: Statement,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Stream<'a>, DbErr>> + 'a + Send>> {
        self.sea_connection.stream_raw(stmt)
    }
}
