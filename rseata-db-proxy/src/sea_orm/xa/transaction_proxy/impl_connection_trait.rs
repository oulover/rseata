use crate::sea_orm::xa::transaction_proxy::{TransactionType, XATransactionProxy};
use sea_orm::{ConnectionTrait, DbBackend, DbErr, ExecResult, QueryResult, Statement};

#[async_trait::async_trait]
impl ConnectionTrait for XATransactionProxy {
    fn get_database_backend(&self) -> DbBackend {
        self.xa_connection_proxy.get_database_backend()
    }
    async fn execute_raw(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
      // let r =  match &self.transaction_type {
      //       TransactionType::Local(local) => {
      //           local.lock().await.as_ref().unwrap().execute_raw(stmt).await
      //       }
      //       TransactionType::XA(xa) => {
      //           Result::Err(DbErr::Custom("".to_string()))
      //       }
      //   };
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
