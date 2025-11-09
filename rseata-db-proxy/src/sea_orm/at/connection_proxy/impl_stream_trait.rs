use crate::sea_orm::at::connection_proxy::ATConnectionProxy;
use sea_orm::{DbBackend, DbErr, QueryStream, Statement, StreamTrait};
use std::pin::Pin;

#[async_trait::async_trait]
impl StreamTrait for ATConnectionProxy {
    type Stream<'a> = QueryStream;

    fn get_database_backend(&self) -> DbBackend {
        self.sea_conn.get_database_backend()
    }

    fn stream_raw<'a>(
        &'a self,
        stmt: Statement,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Stream<'a>, DbErr>> + 'a + Send>> {
        self.sea_conn.stream_raw(stmt)
    }
}
