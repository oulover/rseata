use crate::sea_orm::at::connection_proxy::ATConnectionProxy;
use sea_orm::{ConnectionTrait, DbBackend, DbErr, ExecResult, QueryResult, Statement};

#[async_trait::async_trait]
impl ConnectionTrait for ATConnectionProxy {
    fn get_database_backend(&self) -> DbBackend {
        self.sea_conn.get_database_backend()
    }

    async fn execute_raw(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        println!("------execute_raw----------------------");
        self.sea_conn.execute_raw(stmt).await
    }

    #[allow(unused_variables)]
    async fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr> {
        println!("------execute_unprepared----------------------");
        self.sea_conn.execute_unprepared(sql).await
    }

    async fn query_one_raw(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        println!("------query_one_raw----------------------");
        self.sea_conn.query_one_raw(stmt).await
    }

    async fn query_all_raw(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        println!("------query_all_raw----------------------");
        self.sea_conn.query_all_raw(stmt).await
    }
}
