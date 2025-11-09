use crate::sea_orm::connection_proxy::ConnectionProxy;
use sea_orm::{ConnectionTrait, DbBackend, DbErr, ExecResult, QueryResult, Statement};

#[async_trait::async_trait]
impl ConnectionTrait for ConnectionProxy {
    fn get_database_backend(&self) -> DbBackend {
        println!("------get_database_backend----------------------");
        self.inner.get_database_backend()
    }

    async fn execute_raw(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        println!("------execute_raw----------------------");
        self.inner.execute_raw(stmt).await
    }

    #[allow(unused_variables)]
    async fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr> {
        println!("------execute_unprepared----------------------");
        self.inner.execute_unprepared(sql).await
    }

    async fn query_one_raw(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        println!("------query_one_raw----------------------");
        self.inner.query_one_raw(stmt).await
    }

    async fn query_all_raw(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        println!("------query_all_raw----------------------");
        self.inner.query_all_raw(stmt).await
    }
}
