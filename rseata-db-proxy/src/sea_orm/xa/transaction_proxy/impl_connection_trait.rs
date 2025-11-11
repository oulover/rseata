use crate::sea_orm::xa::transaction_proxy::XATransactionProxy;
use sea_orm::{ConnectionTrait, DbBackend, DbErr, ExecResult, QueryResult, Statement};

#[async_trait::async_trait]
impl ConnectionTrait for XATransactionProxy {
    fn get_database_backend(&self) -> DbBackend {
        ConnectionTrait::get_database_backend(&self.xa_connection_proxy)
    }
    async fn execute_raw(&self, stmt: Statement) -> Result<ExecResult, DbErr> {

        self.xa_connection_proxy.execute_raw(stmt).await
    }
    async fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr> {
        self.xa_connection_proxy.execute_unprepared(sql).await
    }

    async fn query_one_raw(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        self.xa_connection_proxy.query_one_raw(stmt).await
    }

    async fn query_all_raw(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        self.xa_connection_proxy.query_all_raw(stmt).await
    }
}
