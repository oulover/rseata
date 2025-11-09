use sea_orm::{error::*, AccessMode, ConnectOptions, ConnectionTrait, Database,
              DatabaseConnection, DatabaseTransaction, DbBackend, ExecResult,
              IsolationLevel, QueryResult, QueryStream, Statement, StreamTrait,
              TransactionError, TransactionTrait};
use std::ops::Deref;
use sea_orm::sqlx::{Column, Row};
use sqlparser::ast::Value;

use sqlparser::dialect::{GenericDialect, MySqlDialect};
use sqlparser::parser::Parser;


#[derive(Clone, Debug)]
  struct DatabaseConnectionDemo(DatabaseConnection);
impl DatabaseConnectionDemo {
    pub fn new(db: DatabaseConnection) -> Self {
        DatabaseConnectionDemo(db)
    }

    pub async fn connect<C>(opt: C) -> Result<DatabaseConnectionDemo, DbErr>
    where
        C: Into<ConnectOptions>,
    {
        Ok(DatabaseConnectionDemo(Database::connect(opt).await?))
    }
}
impl Deref for DatabaseConnectionDemo {
    type Target = DatabaseConnection;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


impl Default for DatabaseConnectionDemo {
    fn default() -> Self {
        Self(DatabaseConnection::default())
    }
}

impl DatabaseConnectionDemo {

    // async fn query_as_json(&self, sql: &str) -> Result<serde_json::Value, DbErr> {
    //     let results = self.0.query_all(Statement::from_string(self.get_database_backend(), sql)).await?;
    //     let mut rows = Vec::new();
    //     for result in results {
    //         let mut map = serde_json::Map::new();
    //         // for (col, value) in result.column_names() {
    //         //     map.insert(col.to_string(), serde_json::to_value(value).unwrap());
    //         // }
    //         for column_name in result..column_names() {
    //             result.ge
    //             map.insert(col.to_string(), serde_json::to_value(value).unwrap());
    //         }
    //         result.
    //         rows.push(serde_json::Value::Object(map));
    //     }
    //     Ok(serde_json::Value::Array(rows))
    // }


    async fn query_as_json2(&self, sql: &str) -> Result<serde_json::Value, DbErr> {
        // 执行查询
        let stmt = Statement::from_string(self.0.get_database_backend(), sql.to_owned());
        let query_results = self.0.query_all_raw(stmt).await?;


        // 处理结果集
        let mut rows = Vec::new();
        for result in query_results {
            let row = result.try_as_mysql_row().unwrap();
            for my_clo in row.columns(){
                let mut map = serde_json::Map::new();
                // for (col, value) in my_clo.iter() {
                //     map.insert(col.to_string(), serde_json::to_value(value).unwrap());
                // }
                // rows.push(serde_json::Value::Object(map));

                println!("query_as_json -------{:?}",my_clo);
            }
        }

        Ok(serde_json::Value::Array(rows))
    }

    async fn query_as_json(&self, sql: &str) -> Result<serde_json::Value, DbErr> {
        // 执行查询
        let stmt = Statement::from_string(self.0.get_database_backend(), sql.to_owned());
        let query_results = self.query_all_raw(stmt).await?;

        // 处理结果集
        let mut rows = Vec::new();
        for result in query_results {
            let row = result.try_as_mysql_row().ok_or_else(|| {
                DbErr::Custom("Not a MySQL row".into())
            })?;

            let mut map = serde_json::Map::new();

            // 遍历所有列
            for col in row.columns().iter() {
                // 将UStr转换为字符串
                let col_name_str = col.name().to_string();
                let index = col.ordinal();
                // 获取值并转换为JSON类型
                let value = match row.try_get::<String, _>(index) {
                    Ok(s) => serde_json::Value::String(s),
                    Err(_) => match row.try_get::<i64, _>(index) {
                        Ok(n) => serde_json::Value::Number(n.into()),
                        Err(_) => match row.try_get::<f64, _>(index) {
                            Ok(f) => serde_json::Value::from(f),
                            Err(_) => match row.try_get::<bool, _>(index) {
                                Ok(b) => serde_json::Value::Bool(b),
                                Err(_) => serde_json::Value::Null,
                            },
                        },
                    },
                };

                map.insert(col_name_str, value);
            }

            rows.push(serde_json::Value::Object(map));
        }

        Ok(serde_json::Value::Array(rows))
    }


    async fn process_update(&self, update: &sqlparser::ast::Statement) -> Result<(), DbErr> {
        if let sqlparser::ast::Statement::Update {
            table,
            assignments, from, selection, returning,
            or, ..
        } = update {
            let table_name = table.relation.to_string();
            let where_clause = selection.as_ref().map(|e| e.to_string()).unwrap_or_default();

            for assignment in assignments {
                println!("assignments-------------{:?}",assignment);
            }
            let vec_str:Vec<String> = assignments.iter().map(|it|it.target.to_string()).collect();

            println!("assignments-----CLO--------{:?}",vec_str);

            // 获取before_image
            let select_sql = format!("SELECT * FROM {} WHERE {}", table_name, where_clause);
            println!("before select_sql-------{}",select_sql);
            // let before_result =  self.0.execute_unprepared(select_sql.as_ref()).await?;
            let before = self.query_as_json(&select_sql).await;
            match &before {
                Ok(data) => {
                    let old = serde_json::to_string(&data).unwrap_or_default();
                    println!("before old-------{}",old);

                    // 生成回滚sql
                    // fn generate_update_rollback(table: &str, data: &Value) -> String {
                    //     let mut sql = format!("UPDATE {} SET ", table);
                    //     if let Some(first_row) = data.as_array().and_then(|a| a.first()) {
                    //         for (key, value) in first_row.as_object().unwrap() {
                    //             sql.push_str(&format!("{} = {}, ", key, value));
                    //         }
                    //         sql.truncate(sql.len() - 2);
                    //         sql.push_str(" WHERE ..."); // 根据主键生成条件
                    //     }
                    //     sql
                    // }
                    //

                    let mut sql = format!("UPDATE {} SET ", table);
                    if let Some(first_row) = data.as_array().and_then(|a| a.first()) {
                        for (key, value) in first_row.as_object().unwrap() {
                            if vec_str.contains(key) {
                                sql.push_str(&format!("{} = {}, ", key, value));
                            }
                        }
                        sql.truncate(sql.len() - 2);
                        sql.push_str(format!(" WHERE {}",where_clause).as_str()); // 根据主键生成条件
                    }
                    println!("---back sql-------{}",sql);
                    let back = self.0.execute_unprepared(sql.as_str()).await;
                    println!("---back sql--back-----{:?}",back);
                }
                Err(e) => {
                    eprintln!("{}", e);
                }
            }

            println!("before is-------{:?}",before);


            // let r =  self.0.query_all(Statement::from_string(self.0.get_database_backend(), select_sql.as_str())).await?;
            //
            //   println!("before is-------{:?}",r);

            // let before = self.query_as_json(&select_sql).await?;

            // // 执行更新
            //  let _ = self.0.execute_unprepared(select_sql).await?;
            //
            // // 获取after_image
            // let after = self.query_as_json(&select_sql).await?;
            //
            // // 生成回滚SQL
            // let rollback = generate_update_rollback(&table_name, &before);



        }



        Ok(())
    }
}


#[async_trait::async_trait]
impl ConnectionTrait for DatabaseConnectionDemo {
    fn get_database_backend(&self) -> DbBackend {
        self.0.get_database_backend()
    }

    async fn execute_raw(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        self.0.execute_raw(stmt).await
    }





    async fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr> {
        println!("Proxy--sql:--{sql}");


        let dialect = MySqlDialect {};

        let ast = Parser::parse_sql(&dialect, sql);

        match &ast {
            Ok(statements) => {
                for statement in statements {
                    if let sqlparser::ast::Statement::Update{ table: _, assignments: _, from: _, selection: _, returning: _, or: _, .. } = statement {
                        // 记录 Update 的 before_image
                        let r = self.process_update(&statement).await;
                        match r {
                            Ok(_) => {}
                            Err(e) => {eprintln!("process_update error: {}", e)}
                        }

                    } else if let sqlparser::ast::Statement::Insert(i) = statement {
                        // Insert没有 before_image
                    } else if let sqlparser::ast::Statement::Delete(i) = statement {
                        // 记录 Delete 的 before_image
                    }

                    println!("{:#?}", statement);
                }
            }
            Err(e) => eprintln!("Parse error: {}", e),
        }
        let execute_result = self.0.execute_unprepared(sql).await;

        //记录 after_image_build

        //生成 rollback_sql

        execute_result
    }

    async fn query_one_raw(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        todo!()
    }

    async fn query_all_raw(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        todo!()
    }
}

