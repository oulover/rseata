use crate::sea_orm::at::connection_proxy::ATConnectionProxy;
use crate::sea_orm::at::undo_log::DatabaseUndoLogManager;
use async_trait::async_trait;
use rseata_core::branch::branch_transaction::BranchTransaction;
use rseata_core::branch::undo_log::{RowImage, SQLType, UndoLog, UndoLogManager};
use rseata_core::branch::{BranchId, BranchStatus, BranchType};
use rseata_core::types::{ResourceId, Xid};
use sea_orm::{ConnectionTrait, Statement};

#[async_trait]
impl BranchTransaction for ATConnectionProxy {
    async fn branch_commit(
        &self,
        _branch_type: BranchType,
        _xid: Xid,
        branch_id: BranchId,
        _resource_id: ResourceId,
        _application_data: String,
    ) -> anyhow::Result<BranchStatus> {
        tracing::info!("PhaseTwoCommitted branch_commit for branch {}", branch_id.0);

        // 删除该分支的undo logs
        let undo_log_manager = self.undo_log_manager();
        undo_log_manager.delete_undo_logs(branch_id).await?;

        Ok(BranchStatus::PhaseTwoCommitted)
    }

    async fn branch_rollback(
        &self,
        _branch_type: BranchType,
        _xid: Xid,
        branch_id: BranchId,
        _resource_id: ResourceId,
        _application_data: String,
    ) -> anyhow::Result<BranchStatus> {
        tracing::info!("PhaseTwoRollbacked branch_rollback for branch {}", branch_id.0);

        // 获取undo log管理器
        let undo_log_manager = self.undo_log_manager();

        // 获取该分支的所有undo logs
        let undo_logs = undo_log_manager.get_undo_logs(branch_id).await?;
        tracing::info!("Found {} undo logs to rollback", undo_logs.len());

        // 按相反顺序处理（后进先出）
        for undo_log in undo_logs.iter().rev() {
            self.compensate_undo_log(undo_log).await?;
        }

        // 删除已处理的undo logs
        undo_log_manager.delete_undo_logs(branch_id).await?;

        Ok(BranchStatus::PhaseTwoRollbacked)
    }
}

impl ATConnectionProxy {
    /// 根据undo log执行补偿SQL
    async fn compensate_undo_log(&self, undo_log: &UndoLog) -> anyhow::Result<()> {
        match undo_log.sql_type {
            SQLType::UPDATE => self.compensate_update(undo_log).await,
            SQLType::INSERT => self.compensate_insert(undo_log).await,
            SQLType::DELETE => self.compensate_delete(undo_log).await,
        }
    }

    /// 补偿UPDATE操作：将数据还原到before_image状态
    async fn compensate_update(&self, undo_log: &UndoLog) -> anyhow::Result<()> {
        let Some(before_image) = &undo_log.before_image else {
            tracing::warn!("No before_image for UPDATE undo log, skipping");
            return Ok(());
        };

        // 生成UPDATE SQL还原数据
        let sql = self.generate_update_sql(&undo_log.table_name, before_image);
        tracing::debug!("Compensating UPDATE with SQL: {}", sql);

        self.execute_unprepared(&sql).await
            .map_err(|e| anyhow::anyhow!("Failed to execute compensation UPDATE: {}", e))?;

        Ok(())
    }

    /// 补偿INSERT操作：删除插入的数据
    async fn compensate_insert(&self, undo_log: &UndoLog) -> anyhow::Result<()> {
        // INSERT操作通常有after_image（插入的数据）
        // 如果没有after_image，无法补偿
        let Some(after_image) = &undo_log.after_image else {
            tracing::warn!("No after_image for INSERT undo log, cannot compensate");
            return Ok(());
        };

        // 生成DELETE SQL删除插入的数据
        let sql = self.generate_delete_sql(&undo_log.table_name, after_image);
        tracing::debug!("Compensating INSERT with SQL: {}", sql);

        self.execute_unprepared(&sql).await
            .map_err(|e| anyhow::anyhow!("Failed to execute compensation DELETE: {}", e))?;

        Ok(())
    }

    /// 补偿DELETE操作：重新插入删除的数据
    async fn compensate_delete(&self, undo_log: &UndoLog) -> anyhow::Result<()> {
        let Some(before_image) = &undo_log.before_image else {
            tracing::warn!("No before_image for DELETE undo log, skipping");
            return Ok(());
        };

        // 生成INSERT SQL重新插入数据
        let sql = self.generate_insert_sql(&undo_log.table_name, before_image);
        tracing::debug!("Compensating DELETE with SQL: {}", sql);

        self.execute_unprepared(&sql).await
            .map_err(|e| anyhow::anyhow!("Failed to execute compensation INSERT: {}", e))?;

        Ok(())
    }

    /// 生成UPDATE SQL语句
    fn generate_update_sql(&self, table_name: &str, row_image: &RowImage) -> String {
        let mut sql = format!("UPDATE {} SET ", table_name);

        // 构建SET子句
        let set_clauses: Vec<String> = row_image.columns.iter()
            .zip(row_image.values.iter())
            .map(|(col, value)| format!("{} = {}", col, self.format_sql_value(value)))
            .collect();

        sql.push_str(&set_clauses.join(", "));

        // 构建WHERE子句（使用所有列作为条件以确保精确匹配）
        let where_clauses: Vec<String> = row_image.columns.iter()
            .zip(row_image.values.iter())
            .map(|(col, value)| format!("{} = {}", col, self.format_sql_value(value)))
            .collect();

        sql.push_str(" WHERE ");
        sql.push_str(&where_clauses.join(" AND "));

        sql
    }

    /// 生成INSERT SQL语句
    fn generate_insert_sql(&self, table_name: &str, row_image: &RowImage) -> String {
        let columns = row_image.columns.join(", ");
        let values: Vec<String> = row_image.values.iter()
            .map(|value| self.format_sql_value(value))
            .collect();
        let values_str = values.join(", ");

        format!("INSERT INTO {} ({}) VALUES ({})", table_name, columns, values_str)
    }

    /// 生成DELETE SQL语句
    fn generate_delete_sql(&self, table_name: &str, row_image: &RowImage) -> String {
        let where_clauses: Vec<String> = row_image.columns.iter()
            .zip(row_image.values.iter())
            .map(|(col, value)| format!("{} = {}", col, self.format_sql_value(value)))
            .collect();

        format!("DELETE FROM {} WHERE {}", table_name, where_clauses.join(" AND "))
    }

    /// 格式化SQL值
    fn format_sql_value(&self, value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::Null => "NULL".to_string(),
            serde_json::Value::String(s) => format!("'{}'", s.replace("'", "''")),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => if *b { "TRUE".to_string() } else { "FALSE".to_string() },
            _ => format!("'{}'", value.to_string().replace("'", "''")),
        }
    }
}
