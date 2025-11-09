mod impl_connection_trait;
mod impl_stream_trait;
mod impl_transaction_session;
mod impl_transaction_trait;

use crate::sea_orm::transaction_proxy::impl_connection_trait::get_sql_pars_detect;
use rseata_core::branch::branch_manager_outbound::BranchManagerOutbound;
use rseata_core::branch::BranchType;
use rseata_core::resource::Resource;
use rseata_core::RSEATA_CLIENT_SESSION;
use rseata_rm::RSEATA_RM;
use sea_orm::sqlx::{Column, Row, TypeInfo};
use sea_orm::{
    ConnectionTrait, DbErr, Statement,
};
use std::collections::HashMap;

pub struct TransactionProxy {
    inner: sea_orm::DatabaseTransaction,
}
impl TransactionProxy {
    pub(crate) fn new(inner: sea_orm::DatabaseTransaction) -> Self {
        Self { inner }
    }
}

impl std::fmt::Debug for TransactionProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DatabaseTransaction")
    }
}

impl TransactionProxy {
    pub(self) async fn prepare_undo_log(&self) -> Result<(), DbErr> {
        let session = RSEATA_CLIENT_SESSION.try_get().ok();
        println!(
            "TransactionSession------prepare_undo_log----------------------{:?}",
            session
        );
        Ok(())
    }
    pub async fn branch_register(&self) -> Result<(), DbErr> {
        let session = RSEATA_CLIENT_SESSION.try_get().ok();
        println!(
            "TransactionSession------branch_register----------------------{:?}",
            session
        );
        if let Some(session) = &session {
            // 注册 RM 分支事务
            println!("------------注册 RM 分支事务--ing---");
            let xid_guard = session.get_xid();
            if let Some(xid) = xid_guard {
                let lock_keys = session.get_branch_luck_keys().await.unwrap_or_default();
                let branch_id = RSEATA_RM
                    .branch_register(
                        RSEATA_RM.resource_info.get_branch_type().await,
                        RSEATA_RM.resource_info.get_resource_id().await,
                        RSEATA_RM.resource_info.get_client_id().await,
                        xid,
                        "application_data".into(),
                        lock_keys,
                    )
                    .await
                    .map_err(|e| DbErr::Custom(e.to_string()))?;
                println!("------------注册 RM 分支事务---完成{}", branch_id);
                session.set_branch_id(branch_id);
            }
        }
        Ok(())
    }

    pub async fn global_commit(local_commit_result: Result<(), DbErr>) -> Result<(), DbErr> {
        let session = RSEATA_CLIENT_SESSION.try_get().ok();
        println!("TransactionSession------commit----------------------");
        if let Some(session) = session {
            if session.is_global_tx_started() {
                if let Some(xid) = session.get_xid() {
                    let branch_status = match local_commit_result {
                        Ok(_) => rseata_core::branch::BranchStatus::PhaseOneDone,
                        Err(_) => rseata_core::branch::BranchStatus::PhaseOneFailed,
                    };
                    RSEATA_RM
                        .branch_report(
                            BranchType::AT,
                            xid,
                            session.get_branch_id(),
                            branch_status,
                            String::from(""),
                        )
                        .await
                        .map_err(|e| DbErr::Custom(e.to_string()))?;
                }
            }
        }
        local_commit_result
    }

    pub async fn global_rollback() -> Result<(), DbErr> {
        let session = RSEATA_CLIENT_SESSION.try_get().ok();
        println!(
            "TransactionSession------rollback----------------------{:?}",
            session
        );
        if let Some(session) = session {
            if session.is_global_tx_started() {
                if let Some(xid) = session.get_xid() {
                    let branch_status = rseata_core::branch::BranchStatus::PhaseOneFailed;
                    RSEATA_RM
                        .branch_report(
                            BranchType::AT,
                            xid,
                            session.get_branch_id(),
                            branch_status,
                            String::from(""),
                        )
                        .await
                        .map_err(|e| DbErr::Custom(e.to_string()))?;
                }
            }
        }
        Ok(())
    }

    pub async fn check_luck(&self) -> Result<bool, DbErr> {
        let session = RSEATA_CLIENT_SESSION.try_get().ok();
        if let Some(session) = &session {
            let xid_guard = session.get_xid();
            if let Some(xid) = xid_guard {
                let lock_keys = session.get_branch_luck_keys().await.unwrap_or_default();
                let locked = RSEATA_RM
                    .lock_query(
                        RSEATA_RM.resource_info.get_branch_type().await,
                        RSEATA_RM.resource_info.get_resource_id().await,
                        xid,
                        lock_keys,
                    )
                    .await
                    .map_err(|e| DbErr::Custom(e.to_string()))?;
                println!("------------check_luck---完成{}", locked);
                return Ok(locked);
            }
        }
        Ok(true)
    }
}

impl TransactionProxy {
    async fn query_as_json(&self, sql: &str) -> Result<serde_json::Value, DbErr> {
        // 执行查询
        let mut stmt = Statement::from_string(
            ConnectionTrait::get_database_backend(&self.inner),
            sql.to_owned(),
        );
        let query_results = self.query_all_raw(stmt).await?;

        // 处理结果集
        let mut rows = Vec::new();
        for result in query_results {
            let row = result
                .try_as_mysql_row()
                .ok_or_else(|| DbErr::Custom("Not a MySQL row".into()))?;

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

    async fn process_execute(&self, statement: &Statement) -> Result<(), DbErr> {
        println!("Processing execute: {:?}", statement);
        let detect = get_sql_pars_detect(&ConnectionTrait::get_database_backend(&self.inner));
        let parsed = sqlparser::parser::Parser::parse_sql(detect.as_ref(), statement.sql.as_str());

        match &parsed {
            Ok(parsed_statements) => {
                for parsed_statement in parsed_statements {
                    if let sqlparser::ast::Statement::Update {
                        table,
                        assignments,
                        from,
                        selection,
                        returning,
                        or,
                        limit,
                    } = parsed_statement
                    {
                        println!("sqlparser table----: {:?}", table);
                        println!("sqlparser assignments----: {:?}", assignments);
                        println!("sqlparser from----: {:?}", from);
                        println!("sqlparser selection----: {:?}", selection);
                        println!("sqlparser returning----: {:?}", returning);
                        println!("sqlparser or----: {:?}", or);
                        println!("sqlparser limit----: {:?}", limit);

                        let table_name = table.relation.to_string();
                        let where_clause = selection
                            .as_ref()
                            .map(|e| e.to_string())
                            .unwrap_or_default();

                        println!(
                            "sqlparser where_clause----where_clause : {:?}",
                            where_clause
                        );

                        for assignment in assignments {
                            println!("assignments-------------{:?}", assignment);
                        }
                        let vec_str: Vec<String> =
                            assignments.iter().map(|it| it.target.to_string()).collect();

                        println!("assignments-----CLO--------{:?}", vec_str);

                        // 获取before_image
                        let mut before_image_select_sql = {
                            if where_clause.trim().is_empty() {
                                format!("SELECT * FROM {} ", table_name)
                            } else {
                                format!("SELECT * FROM {} WHERE {}", table_name, where_clause)
                            }
                        };
                        println!(
                            "before_image_select_sql is ------ {}",
                            before_image_select_sql
                        );

                        // todo : 把 before_image_select_sql 中的参数 替换为 statement.values 中对应的参数值

                        if !where_clause.trim().is_empty() {
                            if let Some(mut values) = statement.values.clone() {
                                values.0.reverse();

                                for value in values.0.iter() {
                                    if before_image_select_sql.contains("?") {
                                        let index = before_image_select_sql.find("?").unwrap();
                                        before_image_select_sql.replace_range(
                                            index..index + 1,
                                            value.to_string().as_str(),
                                        );
                                    }
                                }

                                println!(
                                    "before_image_select_sql ---EEENNND-- {:?}",
                                    before_image_select_sql
                                );

                                for (param_flag, value) in assignments.iter().zip(values.iter()) {
                                    println!(
                                        "params--------------{:?}----{:?}",
                                        param_flag,
                                        value.to_string()
                                    );
                                }

                                // todo : 把 before_image_select_sql 中的参数 替换为 statement.values 中对应的参数值
                                let select_before = Statement::from_sql_and_values(
                                    ConnectionTrait::get_database_backend(&self.inner),
                                    before_image_select_sql.clone(),
                                    values.clone(), // 这里直接使用原始参数是不对的
                                );

                                let before_values = self.inner.query_all_raw(select_before).await;
                                println!("r--select_before------------{:?}", before_values);

                                if let Ok(old_values) = &before_values {
                                    let key_sql = format!(
                                        "SHOW KEYS FROM {} WHERE Key_name = 'PRIMARY'",
                                        table_name
                                    );
                                    let key_select = Statement::from_string(
                                        ConnectionTrait::get_database_backend(&self.inner),
                                        key_sql,
                                    );
                                    let key_select = self.inner.query_all_raw(key_select).await;
                                    if let Ok(key_select) = key_select {
                                        let keys: Vec<_> = key_select
                                            .iter()
                                            .filter_map(|key| {
                                                key.try_get::<String>("", "Column_name").ok()
                                            })
                                            .collect();

                                        let mut key_values = HashMap::new();
                                        keys.iter().for_each(|key| {
                                            let values = old_values
                                                .iter()
                                                .filter_map(|old_value| {
                                                    let r =
                                                        old_value.try_get_by::<i64, &str>(key).ok();
                                                    println!("----try_get_by--{}-{:?}", key, r);
                                                    r.map(|r| r.to_string())
                                                })
                                                .collect::<Vec<_>>();
                                            key_values.insert(key.to_string(), values);
                                        });

                                        old_values.iter().for_each(|old_one| {
                                            old_one
                                                .try_as_mysql_row()
                                                .unwrap()
                                                .columns()
                                                .iter()
                                                .for_each(|column| {
                                                    let r = column.type_info();
                                                    println!(
                                                        "----type_info--------------{:?}",
                                                        r.name()
                                                    );
                                                })
                                        });

                                        let key_str = key_values
                                            .iter()
                                            .map(|(key, values)| {
                                                format!("{}:{}", key, values.join("_"))
                                            })
                                            .collect::<Vec<String>>()
                                            .join(",");

                                        let session = RSEATA_CLIENT_SESSION.try_get().ok();
                                        if let Some(session) = session {
                                            session.set_branch_luck_keys(key_str).await;
                                        }
                                    }
                                }
                            }
                        }

                        // let before_result =  self.0.execute_unprepared(select_sql.as_ref()).await?;
                        let before = self.query_as_json(&before_image_select_sql).await;
                        match &before {
                            Ok(data) => {
                                let old = serde_json::to_string(&data).unwrap_or_default();
                                println!("before old-------{}", old);

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
                                    sql.push_str(format!(" WHERE {}", where_clause).as_str()); // 根据主键生成条件
                                }
                                println!("---back sql-------{}", sql);
                                let back = self.inner.execute_unprepared(sql.as_str()).await;
                                println!("---back sql--back-----{:?}", back);
                            }
                            Err(e) => {
                                eprintln!("{}", e);
                            }
                        }

                        println!("before is-------{:?}", before);
                    } else if let sqlparser::ast::Statement::Insert(i) = parsed_statement {
                        // Insert没有 before_image
                    } else if let sqlparser::ast::Statement::Delete(i) = parsed_statement {
                        // 记录 Delete 的 before_image
                    }

                    println!("{:#?}", statement);
                }
            }
            Err(e) => eprintln!("Parse error: {}", e),
        }

        Ok(())
    }

    async fn process_luck_keys(&self, update: &sqlparser::ast::Statement) -> Result<(), DbErr> {
        if let sqlparser::ast::Statement::Update {
            table,
            assignments,
            from,
            selection,
            returning,
            or,
            ..
        } = update
        {
            let table_name = table.relation.to_string();
            let where_clause = selection
                .as_ref()
                .map(|e| e.to_string())
                .unwrap_or_default();

            for assignment in assignments {
                println!("assignments-------------{:?}", assignment);
            }
            let vec_str: Vec<String> = assignments.iter().map(|it| it.target.to_string()).collect();

            println!("assignments-----CLO--------{:?}", vec_str);

            // 获取before_image
            let select_sql = format!("SELECT * FROM {} WHERE {}", table_name, where_clause);
            println!("before select_sql-------{}", select_sql);
            // let before_result =  self.0.execute_unprepared(select_sql.as_ref()).await?;
            let before = self.query_as_json(&select_sql).await;
            match &before {
                Ok(data) => {
                    let old = serde_json::to_string(&data).unwrap_or_default();
                    println!("before old-------{}", old);

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
                        sql.push_str(format!(" WHERE {}", where_clause).as_str()); // 根据主键生成条件
                    }
                    println!("---back sql-------{}", sql);
                    let back = self.inner.execute_unprepared(sql.as_str()).await;
                    println!("---back sql--back-----{:?}", back);
                }
                Err(e) => {
                    eprintln!("{}", e);
                }
            }

            println!("before is-------{:?}", before);

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

    async fn process_execute_qw(&self, statement: &Statement) -> Result<(), DbErr> {
        println!("Processing execute: {:?}", statement);
        let detect = get_sql_pars_detect(&ConnectionTrait::get_database_backend(&self.inner));
        let parsed = sqlparser::parser::Parser::parse_sql(detect.as_ref(), statement.sql.as_str());

        match &parsed {
            Ok(parsed_statements) => {
                for parsed_statement in parsed_statements {
                    match parsed_statement {
                        sqlparser::ast::Statement::Update {
                            table,
                            assignments,
                            from,
                            selection,
                            returning: _,
                            or: _,
                            ..
                        } => {
                            let table_name = table.relation.to_string();
                            let where_clause = selection
                                .as_ref()
                                .map(|e| e.to_string())
                                .unwrap_or_default();

                            for assignment in assignments {
                                println!("assignments-------------{:?}", assignment);
                            }
                            let vec_str: Vec<String> =
                                assignments.iter().map(|it| it.target.to_string()).collect();

                            println!("assignments-----CLO--------{:?}", vec_str);

                            // 获取before_image
                            let before_image_select_sql = {
                                if where_clause.trim().is_empty() {
                                    format!("SELECT * FROM {} ", table_name)
                                } else {
                                    format!("SELECT * FROM {} WHERE {}", table_name, where_clause)
                                }
                            };
                            println!(
                                "before_image_select_sql is ------ {}",
                                before_image_select_sql
                            );

                            if !where_clause.trim().is_empty() {
                                if let Some(values) = &statement.values {
                                    // 构建参数映射，将原SQL中的参数值按顺序映射到before_image_select_sql中
                                    let mut new_values: Vec<sea_orm::query::Values> = Vec::new();

                                    // 解析before_image_select_sql，统计参数占位符数量
                                    let before_detect = get_sql_pars_detect(
                                        &ConnectionTrait::get_database_backend(&self.inner),
                                    );
                                    let before_parsed = sqlparser::parser::Parser::parse_sql(
                                        before_detect.as_ref(),
                                        &before_image_select_sql,
                                    );

                                    if let Ok(before_statements) = before_parsed {
                                        if let Some(before_statement) = before_statements.first() {
                                            // 获取before_image_sql中的参数数量
                                            let before_param_count =
                                                self.count_params_in_statement(before_statement);

                                            // 根据before_image_sql中参数的位置，从原始values中提取对应参数
                                            // 这里需要更精确的参数映射逻辑
                                            for _ in 0..before_param_count {
                                                // 按顺序从原始values中取值，这里简化处理
                                                // 实际应用中需要根据参数位置精确映射
                                                new_values.extend_from_slice(&[values.clone()]);
                                                break; // 为了防止重复添加，暂时跳出
                                            }

                                            // 重新解析以准确计算参数数量
                                            let param_count =
                                                self.count_placeholders(&before_image_select_sql);
                                            if param_count > 0 && !values.0.is_empty() {
                                                // 如果before_image_sql有参数，则按需复制原始值
                                                for _ in 0..param_count {
                                                    new_values.extend_from_slice(&[values.clone()]);
                                                    break; // 避免无限复制
                                                }
                                            }
                                        }
                                    }

                                    // 为简化，我们直接使用原始values，但需要确保参数数量匹配
                                    // 这里先用原始参数值
                                    let select_before = Statement::from_sql_and_values(
                                        ConnectionTrait::get_database_backend(&self.inner),
                                        before_image_select_sql.clone(),
                                        values.clone(),
                                    );

                                    let r = self.inner.query_all_raw(select_before).await;

                                    println!("r--select_before------------{:?}", r);

                                    // 处理查询结果，生成undo log
                                    if let Ok(query_results) = r {
                                        if !query_results.is_empty() {
                                            let before =
                                                self.query_as_json(&before_image_select_sql).await;
                                            match &before {
                                                Ok(data) => {
                                                    let old = serde_json::to_string(&data)
                                                        .unwrap_or_default();
                                                    println!("before old-------{}", old);

                                                    // 生成回滚sql
                                                    let mut rollback_sql =
                                                        format!("UPDATE {} SET ", table);
                                                    if let Some(first_row) =
                                                        data.as_array().and_then(|a| a.first())
                                                    {
                                                        for (key, value) in
                                                            first_row.as_object().unwrap()
                                                        {
                                                            if vec_str.contains(key) {
                                                                rollback_sql.push_str(&format!(
                                                                    "{} = {}, ",
                                                                    key, value
                                                                ));
                                                            }
                                                        }
                                                        rollback_sql
                                                            .truncate(rollback_sql.len() - 2);
                                                        rollback_sql.push_str(
                                                            format!(" WHERE {}", where_clause)
                                                                .as_str(),
                                                        );
                                                    }
                                                    println!(
                                                        "---rollback sql-------{}",
                                                        rollback_sql
                                                    );

                                                    // 这里应该将回滚SQL和相关信息保存到undo log中
                                                    // 暂时只打印，实际实现需要存储到事务上下文中
                                                }
                                                Err(e) => {
                                                    eprintln!(
                                                        "Error processing before image: {}",
                                                        e
                                                    );
                                                }
                                            }
                                        } else {
                                            // 如果查询没有返回结果，说明WHERE条件不匹配任何记录
                                            println!("No records found for before image query");
                                        }
                                    }
                                } else {
                                    // 如果没有参数值，直接执行查询（可能不适用于参数化查询）
                                    let select_before = Statement::from_string(
                                        ConnectionTrait::get_database_backend(&self.inner),
                                        before_image_select_sql.clone(),
                                    );

                                    let r = self.inner.query_all_raw(select_before).await;
                                    println!("r--select_before------------{:?}", r);
                                }
                            } else {
                                // 如果没有WHERE子句，查询整个表
                                let select_before = Statement::from_string(
                                    ConnectionTrait::get_database_backend(&self.inner),
                                    before_image_select_sql.clone(),
                                );

                                let r = self.inner.query_all_raw(select_before).await;
                                println!("r--select_before------------{:?}", r);
                            }
                        }
                        sqlparser::ast::Statement::Insert(i) => {
                            // Insert没有 before_image
                            println!("Processing INSERT statement");
                        }
                        sqlparser::ast::Statement::Delete(d) => {
                            // 记录 Delete 的 before_image
                            println!("Processing DELETE statement");

                            let table_name = d.from.to_string();
                            let where_clause = d
                                .selection
                                .as_ref()
                                .map(|e| e.to_string())
                                .unwrap_or_default();

                            let before_image_select_sql = {
                                if where_clause.trim().is_empty() {
                                    format!("SELECT * FROM {} ", table_name)
                                } else {
                                    format!("SELECT * FROM {} WHERE {}", table_name, where_clause)
                                }
                            };

                            if let Some(values) = &statement.values {
                                let select_before = Statement::from_sql_and_values(
                                    ConnectionTrait::get_database_backend(&self.inner),
                                    before_image_select_sql.clone(),
                                    values.clone(),
                                );

                                let r = self.inner.query_all_raw(select_before).await;
                                println!("r--select_before_delete------------{:?}", r);
                            } else {
                                let select_before = Statement::from_string(
                                    ConnectionTrait::get_database_backend(&self.inner),
                                    before_image_select_sql.clone(),
                                );

                                let r = self.inner.query_all_raw(select_before).await;
                                println!("r--select_before_delete------------{:?}", r);
                            }
                        }
                        _ => {
                            // 其他SQL语句，暂时不处理
                            println!("Processing other statement type: {:?}", parsed_statement);
                        }
                    }

                    println!("{:#?}", statement);
                }
            }
            Err(e) => eprintln!("Parse error: {}", e),
        }

        Ok(())
    }

    // 辅助方法：计算SQL语句中的参数占位符数量
    fn count_placeholders(&self, sql: &str) -> usize {
        sql.chars().filter(|&c| c == '?').count()
    }

    // 辅助方法：计算语句中的参数数量
    fn count_params_in_statement(&self, stmt: &sqlparser::ast::Statement) -> usize {
        // 这是一个简化的实现，实际需要根据具体语句类型分析
        // 通常SQL中的参数占位符是?号
        match stmt {
            sqlparser::ast::Statement::Query(query) => {
                // 递归分析查询中的参数
                self.count_params_in_query(query)
            }
            sqlparser::ast::Statement::Update { .. }
            | sqlparser::ast::Statement::Insert { .. }
            | sqlparser::ast::Statement::Delete { .. } => {
                // 简单计算SQL字符串中的问号数量
                format!("{:?}", stmt).chars().filter(|&c| c == '?').count()
            }
            _ => 0,
        }
    }

    fn count_params_in_query(&self, query: &sqlparser::ast::Query) -> usize {
        // 简化实现，计算查询中的参数数量
        format!("{:?}", query).chars().filter(|&c| c == '?').count()
    }
}
