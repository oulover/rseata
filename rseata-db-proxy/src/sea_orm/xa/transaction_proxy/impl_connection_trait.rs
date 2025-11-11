use crate::sea_orm::xa::transaction_proxy::{TransactionType, XATransactionProxy};
use sea_orm::{sqlx, ConnectionTrait, DbBackend, DbErr, ExecResult, QueryResult, Statement, Values};
use sea_orm::sqlx::Executor;
use sea_query_sqlx::SqlxValues;

#[async_trait::async_trait]
impl ConnectionTrait for XATransactionProxy {
    fn get_database_backend(&self) -> DbBackend {
        self.xa_connection_proxy.get_database_backend()
    }

    async fn execute_raw(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        match &self.transaction_type {
            TransactionType::Local(local) => {
                let local_guard = local.lock().await;
                if let Some(local_txn) = local_guard.as_ref() {
                    local_txn.execute_raw(stmt).await
                } else {
                    Err(DbErr::Custom("Local transaction not found".to_string()))
                }
            }
            TransactionType::XA(xa_transaction) => {
                let sql = stmt.sql;
                let mut conn = xa_transaction.connection.lock().await;

                // 使用 SqlxValues 来处理参数绑定
                let values = stmt
                    .values
                    .as_ref()
                    .map_or(Values(Vec::new()), |values| values.clone());

                let query = sqlx::query_with(&sql, SqlxValues(values));

                let result = query
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| DbErr::Custom(e.to_string()))?;

                Ok(ExecResult::from(result))
            }
        }
    }

    async fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr> {
        match &self.transaction_type {
            TransactionType::Local(local) => {
                let local_guard = local.lock().await;
                if let Some(local_txn) = local_guard.as_ref() {
                    local_txn.execute_unprepared(sql).await
                } else {
                    Err(DbErr::Custom("Local transaction not found".to_string()))
                }
            }
            TransactionType::XA(xa_transaction) => {
                let mut conn = xa_transaction.connection.lock().await;

                let result = sqlx::query(sql)
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| DbErr::Custom(e.to_string()))?;

                Ok(ExecResult::from(result))
            }
        }
    }

    async fn query_one_raw(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        match &self.transaction_type {
            TransactionType::Local(local) => {
                let local_guard = local.lock().await;
                if let Some(local_txn) = local_guard.as_ref() {
                    local_txn.query_one_raw(stmt).await
                } else {
                    Err(DbErr::Custom("Local transaction not found".to_string()))
                }
            }
            TransactionType::XA(xa_transaction) => {
                let sql = stmt.sql;
                let mut conn = xa_transaction.connection.lock().await;

                let values = stmt
                    .values
                    .as_ref()
                    .map_or(Values(Vec::new()), |values| values.clone());

                let query = sqlx::query_with(&sql, SqlxValues(values));

                let row = query
                    .fetch_optional(&mut *conn)
                    .await
                    .map_err(|e| DbErr::Custom(e.to_string()))?;

                Ok(row.map(QueryResult::from)  )
            }
        }
    }

    async fn query_all_raw(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        match &self.transaction_type {
            TransactionType::Local(local) => {
                let local_guard = local.lock().await;
                if let Some(local_txn) = local_guard.as_ref() {
                    local_txn.query_all_raw(stmt).await
                } else {
                    Err(DbErr::Custom("Local transaction not found".to_string()))
                }
            }
            TransactionType::XA(xa_transaction) => {
                let sql = stmt.sql;
                let mut conn = xa_transaction.connection.lock().await;

                let values = stmt
                    .values
                    .as_ref()
                    .map_or(Values(Vec::new()), |values| values.clone());

                let query = sqlx::query_with(&sql, SqlxValues(values));

                let rows = query
                    .fetch_all(&mut *conn)
                    .await
                    .map_err(|e| DbErr::Custom(e.to_string()))?;

                Ok(rows
                    .into_iter()
                    .map(QueryResult::from)
                    .collect())
            }
        }
    }
}