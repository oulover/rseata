mod impl_connection_trait;
mod impl_stream_trait;
mod impl_transaction_session;
mod impl_transaction_trait;

use crate::sea_orm::at::connection_proxy::ATConnectionProxy;
use crate::sea_orm::at::transaction_proxy::impl_connection_trait::get_sql_pars_detect;
use rseata_core::RSEATA_CLIENT_SESSION;
use rseata_core::branch::BranchId;
use rseata_core::branch::BranchType;
use rseata_core::branch::branch_manager_outbound::BranchManagerOutbound;
use rseata_core::branch::undo_log::{RowImage, SQLType, UndoLog, UndoLogManager};
use rseata_core::resource::Resource;
use rseata_core::types::Xid;
use rseata_rm::RSEATA_RM;
use sea_orm::sqlx::{Column, Row, TypeInfo};
use sea_orm::{ConnectionTrait, DbErr, Statement};
use std::collections::HashMap;
use rseata_core::branch::branch_transaction::BranchTransactionRegistry;
use tokio::sync::Mutex;

pub struct ATTransactionProxy {
    at_connection_proxy: ATConnectionProxy,
    sea_transaction: sea_orm::DatabaseTransaction,
    undo_logs: std::sync::Arc<Mutex<Vec<UndoLog>>>,
}
impl ATTransactionProxy {
    pub(crate) fn new(
        at_connection_proxy: ATConnectionProxy,
        sea_transaction: sea_orm::DatabaseTransaction,
    ) -> Self {
        Self {
            at_connection_proxy,
            sea_transaction,
            undo_logs: std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }
}

impl std::fmt::Debug for ATTransactionProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DatabaseTransaction")
    }
}

impl ATTransactionProxy {
    pub(self) async fn prepare_undo_log(&self) -> Result<(), DbErr> {
        let session = RSEATA_CLIENT_SESSION.try_get().ok();
        println!(
            "TransactionSession------prepare_undo_log----------------------{:?}",
            session
        );

        // Get undo logs collected during transaction execution
        let undo_logs = self.undo_logs.lock().await;

        if undo_logs.is_empty() {
            println!("No undo logs to prepare");
            return Ok(());
        }

        println!("Preparing {} undo logs", undo_logs.len());

        // Get undo log manager from connection
        let undo_log_manager = self.at_connection_proxy.undo_log_manager();

        // Store each undo log
        for undo_log in undo_logs.iter() {
            if let Err(e) = undo_log_manager.add_undo_log(undo_log.clone()).await {
                eprintln!("Failed to store undo log: {}", e);
                // Continue with other logs
            }
        }

        println!("Successfully prepared {} undo logs", undo_logs.len());
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
                let lock_keys = session.get_branch_lock_keys().await.unwrap_or_default();
                let branch_id = RSEATA_RM
                    .branch_transaction_registry(
                        RSEATA_RM.resource_info.get_branch_type().await,
                        RSEATA_RM.resource_info.get_resource_id().await,
                        RSEATA_RM.resource_info.get_client_id().await,
                        xid,
                        "application_data".into(),
                        lock_keys,
                        Box::new(self.at_connection_proxy.clone()),
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

    pub async fn check_lock(&self) -> Result<bool, DbErr> {
        let session = RSEATA_CLIENT_SESSION.try_get().ok();
        if let Some(session) = &session {
            let xid_guard = session.get_xid();
            if let Some(xid) = xid_guard {
                let lock_keys = session.get_branch_lock_keys().await.unwrap_or_default();
                let locked = RSEATA_RM
                    .lock_query(
                        RSEATA_RM.resource_info.get_branch_type().await,
                        RSEATA_RM.resource_info.get_resource_id().await,
                        xid,
                        lock_keys,
                    )
                    .await
                    .map_err(|e| DbErr::Custom(e.to_string()))?;
                return Ok(locked);
            }
        }
        Ok(true)
    }
}

impl ATTransactionProxy {
    async fn query_as_json(&self, sql: &str) -> Result<serde_json::Value, DbErr> {
        // 执行查询
        let mut stmt = Statement::from_string(
            ConnectionTrait::get_database_backend(&self.at_connection_proxy),
            sql.to_owned(),
        );
        let query_results = self.query_all_raw(stmt).await?;

        // 处理结果集
        let mut rows = Vec::new();
        for result in query_results {
            let mut map = serde_json::Map::new();

            // 获取列名
            let column_names = result.column_names();

            for (i, col_name) in column_names.iter().enumerate() {
                // 获取值并转换为JSON类型
                let value = if let Ok(s) = result.try_get::<String>("", col_name) {
                    serde_json::Value::String(s)
                } else if let Ok(n) = result.try_get::<i32>("", col_name) {
                    serde_json::Value::Number(n.into())
                } else if let Ok(n) = result.try_get::<i64>("", col_name) {
                    serde_json::Value::Number(n.into())
                } else if let Ok(f) = result.try_get::<f64>("", col_name) {
                    serde_json::Value::from(f)
                } else if let Ok(b) = result.try_get::<bool>("", col_name) {
                    serde_json::Value::Bool(b)
                } else if let Ok(opt_s) = result.try_get::<Option<String>>("", col_name) {
                    match opt_s {
                        Some(s) => serde_json::Value::String(s),
                        None => serde_json::Value::Null,
                    }
                } else if let Ok(opt_i32) = result.try_get::<Option<i32>>("", col_name) {
                    match opt_i32 {
                        Some(n) => serde_json::Value::Number(n.into()),
                        None => serde_json::Value::Null,
                    }
                } else if let Ok(opt_i64) = result.try_get::<Option<i64>>("", col_name) {
                    match opt_i64 {
                        Some(n) => serde_json::Value::Number(n.into()),
                        None => serde_json::Value::Null,
                    }
                } else if let Ok(opt_f64) = result.try_get::<Option<f64>>("", col_name) {
                    match opt_f64 {
                        Some(f) => serde_json::Value::from(f),
                        None => serde_json::Value::Null,
                    }
                } else if let Ok(opt_bool) = result.try_get::<Option<bool>>("", col_name) {
                    match opt_bool {
                        Some(b) => serde_json::Value::Bool(b),
                        None => serde_json::Value::Null,
                    }
                } else {
                    serde_json::Value::Null
                };

                map.insert(col_name.to_string(), value);
            }

            rows.push(serde_json::Value::Object(map));
        }

        Ok(serde_json::Value::Array(rows))
    }

    async fn process_execute(&self, statement: &Statement) -> Result<(), DbErr> {
        println!("Processing execute: {:?}", statement);

        // 获取当前会话信息
        let session = RSEATA_CLIENT_SESSION.try_get().ok();
        let xid_opt = session.as_ref().and_then(|s| s.get_xid().as_ref().cloned());
        let branch_id_opt = session.as_ref().and_then(|s| {
            // 使用 Option 类型的 get_branch_id
            Some(s.get_branch_id()) // 这会返回 BranchId，我们假设它始终存在
        });

        let (xid_opt, branch_id_opt): (Option<Xid>, Option<BranchId>) = (xid_opt, branch_id_opt);

        // 如果不是全局事务，不需要记录undo log
        if xid_opt.is_none() || branch_id_opt.is_none() {
            println!("Not in global transaction, skipping undo log");
            return Ok(());
        }

        let xid = xid_opt.unwrap();
        let branch_id = branch_id_opt.unwrap();

        let detect = get_sql_pars_detect(&ConnectionTrait::get_database_backend(
            &self.at_connection_proxy,
        ));
        let parsed = sqlparser::parser::Parser::parse_sql(detect.as_ref(), statement.sql.as_str());

        match &parsed {
            Ok(parsed_statements) => {
                for parsed_statement in parsed_statements {
                    match parsed_statement {
                        sqlparser::ast::Statement::Update {
                            table,
                            assignments,
                            selection,
                            ..
                        } => {
                            self.process_update(
                                statement,
                                table,
                                assignments,
                                selection.as_ref(),
                                xid.clone(),  // 克隆xid避免所有权问题
                                branch_id,
                            )
                            .await?;
                        }
                        sqlparser::ast::Statement::Insert(insert) => {
                            self.process_insert_simple(&insert, xid.clone(), branch_id).await?;  // 克隆xid
                        }
                        sqlparser::ast::Statement::Delete(delete) => {
                            self.process_delete_simple(&delete, xid.clone(), branch_id).await?;  // 克隆xid
                        }
                        _ => {
                            // 其他语句类型，不记录undo log
                            println!("Skipping undo log for statement type: {:?}", parsed_statement);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("SQL parse error: {}", e);
                // 解析失败时不记录undo log，但继续执行SQL
            }
        }

        Ok(())
    }

    /// 处理UPDATE语句，捕获before_image和after_image
    async fn process_update(
        &self,
        statement: &Statement,
        table: &sqlparser::ast::TableWithJoins,
        assignments: &[sqlparser::ast::Assignment],
        selection: Option<&sqlparser::ast::Expr>,
        xid: Xid,
        branch_id: BranchId,
    ) -> Result<(), DbErr> {
        use crate::sea_orm::at::undo_log::{create_row_image_from_single_row, create_undo_log};
        use rseata_core::branch::undo_log::SQLType;

        let table_name: String = table.relation.to_string();
        let where_clause: String = selection.map(|e| e.to_string()).unwrap_or_default();

        println!("Processing UPDATE on table: {}, where: {}", table_name, where_clause);

        // 1. 获取before_image
        let before_image_select_sql = if where_clause.trim().is_empty() {
            format!("SELECT * FROM {} ", table_name)
        } else {
            format!("SELECT * FROM {} WHERE {}", table_name, where_clause)
        };

        println!("Before image SQL: {}", before_image_select_sql);

        // 执行查询获取before_image
        let before_stmt = if let Some(values) = &statement.values {
            // 使用原始参数值
            Statement::from_sql_and_values(
                ConnectionTrait::get_database_backend(&self.at_connection_proxy),
                before_image_select_sql.clone(),
                values.clone(),
            )
        } else {
            Statement::from_string(
                ConnectionTrait::get_database_backend(&self.at_connection_proxy),
                before_image_select_sql.clone(),
            )
        };

        let before_results = self.at_connection_proxy.query_all_raw(before_stmt).await?;

        // 为每一行创建undo log
        let mut undo_logs_guard = self.undo_logs.lock().await;
        let mut all_row_images = Vec::new();

        for row in before_results.iter() {
            if let Ok(row_image) = create_row_image_from_single_row(row) {
                all_row_images.push(row_image.clone());

                // 为每一行创建单独的undo log
                let undo_log = create_undo_log(
                    branch_id,
                    xid.clone(),  // 克隆xid避免所有权问题
                    table_name.clone(),
                    SQLType::UPDATE,
                    Some(row_image),
                    None, // UPDATE的after_image在AT模式中通常不保存
                );

                undo_logs_guard.push(undo_log);
                println!("Added UPDATE undo log for row");
            }
        }

        println!("Captured {} before image rows, created {} undo logs",
                 all_row_images.len(), undo_logs_guard.len());

        // 记录锁键（用于全局锁检查）
        if !all_row_images.is_empty() {
            drop(undo_logs_guard); // 释放锁，避免死锁
            self.record_lock_keys(&table_name, &all_row_images).await?;
        }

        Ok(())
    }

    /// 处理INSERT语句，只记录after_image
    async fn process_insert_simple(
        &self,
        insert: &sqlparser::ast::Insert,
        xid: Xid,
        branch_id: BranchId,
    ) -> Result<(), DbErr> {
        use crate::sea_orm::at::undo_log::{create_row_image_from_single_row, create_undo_log};
        use rseata_core::branch::undo_log::SQLType;

        // 从insert语句中获取表名
        let table_name = insert.table.to_string();
        println!("Processing INSERT on table: {}", table_name);

        if let Some(source) = &insert.source {
            match &*source.body {
                sqlparser::ast::SetExpr::Values(values) => {
                    for row_values in &values.rows {
                        let column_names: Vec<String> = insert.columns.iter()
                            .map(|col| col.value.clone())
                            .collect();

                        let mut column_values: Vec<serde_json::Value> = Vec::new();

                        for expr in row_values {
                            let value = self.expr_to_json_value(expr)?;
                            column_values.push(value);
                        }

                        let row_image = RowImage {
                            columns: column_names.clone(),
                            values: column_values,
                        };

                        let undo_log = create_undo_log(
                            branch_id,
                            xid.clone(),  // 克隆xid避免所有权问题
                            table_name.clone(),
                            SQLType::INSERT,
                            None, // INSERT没有before_image
                            Some(row_image), // 保存插入的数据作为after_image
                        );

                        // 添加到undo logs集合
                        let mut undo_logs = self.undo_logs.lock().await;
                        undo_logs.push(undo_log);
                        println!("Added INSERT undo log to collection");
                    }
                }
                // 对于INSERT ... SELECT的情况，处理方式不同
                _ => {
                    // 对于复杂的INSERT语句，暂时只记录基本信息
                    let undo_log = create_undo_log(
                        branch_id,
                        xid.clone(),  // 克隆xid避免所有权问题
                        table_name,
                        SQLType::INSERT,
                        None, // INSERT没有before_image
                        None, // 暂时无法获取插入的数据
                    );

                    let mut undo_logs = self.undo_logs.lock().await;
                    undo_logs.push(undo_log);
                    println!("Added INSERT undo log to collection (complex statement)");
                }
            }
        } else {
            // 如果没有source，记录一个基本的undo日志
            let undo_log = create_undo_log(
                branch_id,
                xid.clone(),  // 克隆xid避免所有权问题
                table_name,
                SQLType::INSERT,
                None, // INSERT没有before_image
                None, // 暂时无法获取插入的数据
            );

            let mut undo_logs = self.undo_logs.lock().await;
            undo_logs.push(undo_log);
            println!("Added INSERT undo log to collection (no source)");
        }

        Ok(())
    }

    /// 处理DELETE语句，记录before_image用于回滚
    async fn process_delete_simple(
        &self,
        delete: &sqlparser::ast::Delete,
        xid: Xid,
        branch_id: BranchId,
    ) -> Result<(), DbErr> {
        use crate::sea_orm::at::undo_log::{create_row_image_from_single_row, create_undo_log};
        use rseata_core::branch::undo_log::SQLType;

        let table_name = delete.from.to_string();
        let where_clause = delete
            .selection
            .as_ref()
            .map(|e| e.to_string())
            .unwrap_or_default();

        println!("Processing DELETE on table: {}, where: {}", table_name, where_clause);

        // 1. 获取before_image（即将被删除的行）
        let before_image_select_sql = if where_clause.trim().is_empty() {
            format!("SELECT * FROM {} ", table_name)
        } else {
            format!("SELECT * FROM {} WHERE {}", table_name, where_clause)
        };

        println!("Before image SQL: {}", before_image_select_sql);

        // 2. 执行查询获取before_image
        let before_stmt = Statement::from_string(
            ConnectionTrait::get_database_backend(&self.at_connection_proxy),
            before_image_select_sql.clone(),
        );

        let before_results = self.at_connection_proxy.query_all_raw(before_stmt).await?;

        // 3. 为每一行创建undo log
        let mut undo_logs_guard = self.undo_logs.lock().await;
        let mut all_row_images = Vec::new();

        for row in before_results.iter() {
            if let Ok(row_image) = create_row_image_from_single_row(row) {
                all_row_images.push(row_image.clone());

                let undo_log = create_undo_log(
                    branch_id,
                    xid.clone(),  // 克隆xid避免所有权问题
                    table_name.clone(),
                    SQLType::DELETE,
                    Some(row_image),
                    None,
                );

                undo_logs_guard.push(undo_log);
                println!("Added DELETE undo log for row");
            }
        }

        println!("Captured {} before image rows, created {} undo logs",
                 all_row_images.len(), undo_logs_guard.len());

        // 4. 记录锁键
        if !all_row_images.is_empty() {
            drop(undo_logs_guard); // 释放锁，避免死锁
            self.record_lock_keys(&table_name, &all_row_images).await?;
        }

        Ok(())
    }

    /// 处理DELETE语句，只记录before_image
    async fn process_delete(
        &self,
        statement: &Statement,
        from: &sqlparser::ast::TableWithJoins,
        selection: Option<&sqlparser::ast::Expr>,
        xid: Xid,
        branch_id: BranchId,
    ) -> Result<(), DbErr> {
        use crate::sea_orm::at::undo_log::{create_row_image_from_single_row, create_undo_log};
        use rseata_core::branch::undo_log::SQLType;

        let table_name = from.relation.to_string();
        let where_clause = selection.map(|e| e.to_string()).unwrap_or_default();

        println!("Processing DELETE on table: {}, where: {}", table_name, where_clause);

        // 1. 获取before_image
        let before_image_select_sql = if where_clause.trim().is_empty() {
            format!("SELECT * FROM {} ", table_name)
        } else {
            format!("SELECT * FROM {} WHERE {}", table_name, where_clause)
        };

        println!("Before image SQL: {}", before_image_select_sql);

        // 执行查询获取before_image
        let before_stmt = if let Some(values) = &statement.values {
            Statement::from_sql_and_values(
                ConnectionTrait::get_database_backend(&self.at_connection_proxy),
                before_image_select_sql.clone(),
                values.clone(),
            )
        } else {
            Statement::from_string(
                ConnectionTrait::get_database_backend(&self.at_connection_proxy),
                before_image_select_sql.clone(),
            )
        };

        let before_results = self.at_connection_proxy.query_all_raw(before_stmt).await?;

        // 为每一行创建undo log
        let mut undo_logs_guard = self.undo_logs.lock().await;
        let mut all_row_images = Vec::new();

        for row in before_results.iter() {
            if let Ok(row_image) = create_row_image_from_single_row(row) {
                all_row_images.push(row_image.clone());

                // 为每一行创建单独的undo log
                let undo_log = create_undo_log(
                    branch_id,
                    xid.clone(),  // 克隆xid避免所有权问题
                    table_name.clone(),
                    SQLType::DELETE,
                    Some(row_image),
                    None,
                );

                undo_logs_guard.push(undo_log);
                println!("Added DELETE undo log for row");
            }
        }

        println!("Captured {} before image rows, created {} undo logs",
                 all_row_images.len(), undo_logs_guard.len());

        // 记录锁键
        if !all_row_images.is_empty() {
            drop(undo_logs_guard); // 释放锁，避免死锁
            self.record_lock_keys(&table_name, &all_row_images).await?;
        }

        Ok(())
    }

    /// 记录锁键（主键值），用于全局锁检查
    async fn record_lock_keys(
        &self,
        table_name: &str,
        row_images: &[RowImage],
    ) -> Result<(), DbErr> {
        use std::collections::HashMap;

        // 查询表的主键
        let key_sql = format!(
            "SHOW KEYS FROM {} WHERE Key_name = 'PRIMARY'",
            table_name
        );
        let key_select = Statement::from_string(
            ConnectionTrait::get_database_backend(&self.at_connection_proxy),
            key_sql,
        );

        let key_results = self.at_connection_proxy.query_all_raw(key_select).await?;
        let primary_keys: Vec<String> = key_results
            .iter()
            .filter_map(|row| row.try_get::<String>("", "Column_name").ok())
            .collect();

        println!("Primary keys for {}: {:?}", table_name, primary_keys);

        if primary_keys.is_empty() {
            // 没有主键，使用所有列作为锁键
            let all_keys: Vec<String> = row_images
                .iter()
                .flat_map(|img| img.columns.clone())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            let key_str = all_keys.join(",");
            let session = RSEATA_CLIENT_SESSION.try_get().ok();
            if let Some(session) = session {
                session.set_branch_lock_keys(key_str).await;
            }
            return Ok(());
        }

        // 提取主键值
        let mut lock_keys_map = HashMap::new();
        for pk in &primary_keys {
            let values: Vec<String> = row_images
                .iter()
                .filter_map(|img| {
                    img.columns
                        .iter()
                        .position(|col| col == pk)
                        .and_then(|idx| {
                            // 将值转换为字符串
                            Some(img.values.get(idx)?.to_string())
                        })
                })
                .collect();
            lock_keys_map.insert(pk.clone(), values);
        }

        // 构建锁键字符串格式: "pk1:value1_value2,pk2:value1_value2"
        let key_str = lock_keys_map
            .iter()
            .map(|(key, values)| format!("{}:{}", key, values.join("_")))
            .collect::<Vec<String>>()
            .join(",");

        println!("Lock keys: {}", key_str);

        // 保存到会话
        let session = RSEATA_CLIENT_SESSION.try_get().ok();
        if let Some(session) = session {
            session.set_branch_lock_keys(key_str).await;
        }

        Ok(())
    }

    /// 将SQL表达式转换为JSON值
    fn expr_to_json_value(&self, expr: &sqlparser::ast::Expr) -> Result<serde_json::Value, DbErr> {
        match expr {
            sqlparser::ast::Expr::Value(value_with_span) => {
                // 为了兼容不同的sqlparser版本，使用更通用的处理方式
                // 将value转换为字符串，再尝试解析为适当的JSON类型
                let value_str = value_with_span.to_string();

                // 尝试判断值的类型
                if value_str.starts_with('\'') || value_str.starts_with('"') {
                    // 字符串类型，去掉引号
                    let cleaned = value_str.trim_matches(|c| c == '\'' || c == '"');
                    Ok(serde_json::Value::String(cleaned.to_string()))
                } else if value_str.eq_ignore_ascii_case("true") {
                    Ok(serde_json::Value::Bool(true))
                } else if value_str.eq_ignore_ascii_case("false") {
                    Ok(serde_json::Value::Bool(false))
                } else if value_str.eq_ignore_ascii_case("null") {
                    Ok(serde_json::Value::Null)
                } else if let Ok(num_val) = value_str.parse::<i64>() {
                    Ok(serde_json::Value::Number(num_val.into()))
                } else if let Ok(num_val) = value_str.parse::<f64>() {
                    // 确保是有效数字
                    if num_val.is_finite() {
                        Ok(serde_json::Value::from(num_val))
                    } else {
                        Ok(serde_json::Value::String(value_str))
                    }
                } else {
                    Ok(serde_json::Value::String(value_str))
                }
            },
            _ => {
                // 对于复杂表达式，将其转换为字符串
                Ok(serde_json::Value::String(expr.to_string()))
            }
        }
    }

    async fn process_lock_keys(&self, update: &sqlparser::ast::Statement) -> Result<(), DbErr> {
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
                    let back = self
                        .at_connection_proxy
                        .execute_unprepared(sql.as_str())
                        .await;
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
        let detect = get_sql_pars_detect(&ConnectionTrait::get_database_backend(
            &self.at_connection_proxy,
        ));
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
                                        &ConnectionTrait::get_database_backend(
                                            &self.at_connection_proxy,
                                        ),
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
                                        ConnectionTrait::get_database_backend(
                                            &self.at_connection_proxy,
                                        ),
                                        before_image_select_sql.clone(),
                                        values.clone(),
                                    );

                                    let r =
                                        self.at_connection_proxy.query_all_raw(select_before).await;

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
                                        ConnectionTrait::get_database_backend(
                                            &self.at_connection_proxy,
                                        ),
                                        before_image_select_sql.clone(),
                                    );

                                    let r =
                                        self.at_connection_proxy.query_all_raw(select_before).await;
                                    println!("r--select_before------------{:?}", r);
                                }
                            } else {
                                // 如果没有WHERE子句，查询整个表
                                let select_before = Statement::from_string(
                                    ConnectionTrait::get_database_backend(
                                        &self.at_connection_proxy,
                                    ),
                                    before_image_select_sql.clone(),
                                );

                                let r = self.at_connection_proxy.query_all_raw(select_before).await;
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
                                    ConnectionTrait::get_database_backend(
                                        &self.at_connection_proxy,
                                    ),
                                    before_image_select_sql.clone(),
                                    values.clone(),
                                );

                                let r = self.at_connection_proxy.query_all_raw(select_before).await;
                                println!("r--select_before_delete------------{:?}", r);
                            } else {
                                let select_before = Statement::from_string(
                                    ConnectionTrait::get_database_backend(
                                        &self.at_connection_proxy,
                                    ),
                                    before_image_select_sql.clone(),
                                );

                                let r = self.at_connection_proxy.query_all_raw(select_before).await;
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
