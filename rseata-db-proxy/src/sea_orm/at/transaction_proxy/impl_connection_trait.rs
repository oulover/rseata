use std::ops::Deref;
use crate::sea_orm::at::transaction_proxy::ATTransactionProxy;
use rseata_core::RSEATA_CLIENT_SESSION;
use sea_orm::sqlx::Row;
use sea_orm::{ConnectionTrait, DbBackend, DbErr, ExecResult, QueryResult, Statement};
use sqlparser::dialect::{Dialect, MySqlDialect, PostgreSqlDialect, SQLiteDialect};
use sqlparser::parser::Parser;

#[async_trait::async_trait]
impl ConnectionTrait for ATTransactionProxy {
    fn get_database_backend(&self) -> DbBackend {
        println!("Transaction------get_database_backend----------------------");
        ConnectionTrait::get_database_backend(&self.at_connection_proxy)
    }

    async fn execute_raw(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        let session = RSEATA_CLIENT_SESSION.try_get().ok();
        println!("Transaction------execute_raw------------{:?}", session);
        self.process_execute(&stmt).await.ok();

        self.at_connection_proxy.execute_raw(stmt).await
    }
    async fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr> {
        println!("Transaction------execute_unprepared----------------------");
        self.at_connection_proxy.execute_unprepared(sql).await
    }

    async fn query_one_raw(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        println!("Transaction------query_one_raw----------------------");
        self.at_connection_proxy.query_one_raw(stmt).await
    }

    async fn query_all_raw(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        println!("Transaction------query_all_raw----------------------");
        self.at_connection_proxy.query_all_raw(stmt).await
    }
}

pub fn get_sql_pars_detect(db_backend: &DbBackend) -> Box<dyn Dialect +Send + 'static> {
    match db_backend {
        DbBackend::MySql => Box::new(MySqlDialect {}),
        DbBackend::Postgres => Box::new(PostgreSqlDialect {}),
        DbBackend::Sqlite => Box::new(SQLiteDialect {}),
        _ => {
            unreachable!()
        }
    }
}



pub struct SqlLog {
    sql: String,
    params: Vec<String>,
    result: Result<QueryResult, DbErr>,
    types: i32, //
}
