use crate::sea_orm::xa::connection_proxy::XAConnectionProxy;
use sea_orm::{ConnectionTrait, DbBackend, DbErr, ExecResult, QueryResult, Statement};

#[async_trait::async_trait]
impl ConnectionTrait for XAConnectionProxy {
    fn get_database_backend(&self) -> DbBackend {
        self.sea_connection.get_database_backend()
    }

    async fn execute_raw(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        self.sea_connection.execute_raw(stmt).await
    }

    async fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr> {
        self.sea_connection.execute_unprepared(sql).await
    }

    async fn query_one_raw(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        self.sea_connection.query_one_raw(stmt).await
    }

    async fn query_all_raw(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        self.sea_connection.query_all_raw(stmt).await
    }
}
