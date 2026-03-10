use async_trait::async_trait;
use rseata_core::branch::undo_log::{RowImage, SQLType, UndoLog, UndoLogManager};
use rseata_core::branch::BranchId;
use rseata_core::types::Xid;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, DatabaseBackend, Statement};
use serde_json;
use std::sync::Arc;

pub struct PostgreSQLErrorsUndoLogManager {
    db: Arc<DatabaseConnection>,
}

impl PostgreSQLErrorsUndoLogManager {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    /// Create undo_log table if it doesn't exist for PostgreSQL
    pub async fn create_table_if_not_exists(&self) -> Result<(), DbErr> {
        // SQL to create undo_log table for PostgreSQL
        let sql = r#"
        CREATE TABLE IF NOT EXISTS undo_log (
            id BIGSERIAL PRIMARY KEY,
            branch_id BIGINT NOT NULL,
            xid VARCHAR(128) NOT NULL,
            table_name VARCHAR(128) NOT NULL,
            sql_type SMALLINT NOT NULL, -- 1: INSERT, 2: UPDATE, 3: DELETE
            before_image JSONB,
            after_image JSONB,
            log_created BIGINT NOT NULL,
            log_modified BIGINT NOT NULL,
            CONSTRAINT ux_undo_log UNIQUE (branch_id, xid)
        );

        CREATE INDEX IF NOT EXISTS idx_branch_id ON undo_log (branch_id);
        CREATE INDEX IF NOT EXISTS idx_xid ON undo_log (xid);
        "#;

        self.db.execute_unprepared(sql).await?;
        Ok(())
    }
}

#[async_trait]
impl UndoLogManager for PostgreSQLErrorsUndoLogManager {
    async fn add_undo_log(&self, undo_log: UndoLog) -> anyhow::Result<()> {
        let before_image_str = undo_log.before_image
            .as_ref()
            .map(|img| serde_json::to_string(img))
            .transpose()
            .map_err(|e| anyhow::anyhow!("Failed to serialize before image: {}", e))?
            .unwrap_or_else(|| "null".to_string());

        let after_image_str = undo_log.after_image
            .as_ref()
            .map(|img| serde_json::to_string(img))
            .transpose()
            .map_err(|e| anyhow::anyhow!("Failed to serialize after image: {}", e))?
            .unwrap_or_else(|| "null".to_string());

        let sql_type_value = match undo_log.sql_type {
            SQLType::INSERT => 1,
            SQLType::UPDATE => 2,
            SQLType::DELETE => 3,
        };

        // Use execute_unprepared with properly escaped values for PostgreSQL
        let sql = format!(
            r#"INSERT INTO undo_log
            (branch_id, xid, table_name, sql_type, before_image, after_image, log_created, log_modified)
            VALUES ({}, '{}', '{}', {}, '{}'::jsonb, '{}'::jsonb, {}, {})"#,
            undo_log.branch_id.0,
            undo_log.xid.0.replace("'", "''"),
            undo_log.table_name.replace("'", "''"),
            sql_type_value,
            before_image_str.replace("'", "''"),
            after_image_str.replace("'", "''"),
            undo_log.log_created,
            undo_log.log_modified
        );

        self.db.execute_unprepared(&sql).await
            .map_err(|e| anyhow::anyhow!("Failed to insert undo log: {}", e))?;

        Ok(())
    }

    async fn get_undo_logs(&self, branch_id: BranchId) -> anyhow::Result<Vec<UndoLog>> {
        let sql = format!(
            "SELECT id, branch_id, xid, table_name, sql_type, before_image, after_image, log_created, log_modified FROM undo_log WHERE branch_id = {}",
            branch_id.0
        );

        let stmt = Statement::from_string(DatabaseBackend::Postgres, sql);
        let rows = self.db.query_all_raw(stmt)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to query undo logs: {}", e))?;

        let mut undo_logs = Vec::new();
        for row in rows {
            let _id: i64 = row.try_get::<i64>("", "id").unwrap_or(0);
            let branch_id_val: i64 = row.try_get::<i64>("", "branch_id").unwrap_or(0);
            let xid_str: String = row.try_get::<String>("", "xid").unwrap_or_default();
            let table_name: String = row.try_get::<String>("", "table_name").unwrap_or_default();
            let sql_type_val: i16 = row.try_get::<i16>("", "sql_type").unwrap_or(0);
            let before_image_json: Option<String> = row.try_get::<Option<String>>("", "before_image").unwrap_or(None);
            let after_image_json: Option<String> = row.try_get::<Option<String>>("", "after_image").unwrap_or(None);
            let log_created: i64 = row.try_get::<i64>("", "log_created").unwrap_or(0);
            let log_modified: i64 = row.try_get::<i64>("", "log_modified").unwrap_or(0);

            // Parse SQL type
            let sql_type = match sql_type_val {
                1 => SQLType::INSERT,
                2 => SQLType::UPDATE,
                3 => SQLType::DELETE,
                _ => continue,
            };

            // Parse before_image
            let before_image = if let Some(json_str) = before_image_json {
                if json_str == "null" || json_str.trim().is_empty() {
                    None
                } else {
                    serde_json::from_str(&json_str).ok()
                }
            } else {
                None
            };

            // Parse after_image
            let after_image = if let Some(json_str) = after_image_json {
                if json_str == "null" || json_str.trim().is_empty() {
                    None
                } else {
                    serde_json::from_str(&json_str).ok()
                }
            } else {
                None
            };

            let undo_log = UndoLog {
                branch_id: BranchId(branch_id_val as u64),
                xid: Xid::from(xid_str.as_str()),
                table_name,
                sql_type,
                before_image,
                after_image,
                log_created,
                log_modified,
            };

            undo_logs.push(undo_log);
        }

        Ok(undo_logs)
    }

    async fn delete_undo_logs(&self, branch_id: BranchId) -> anyhow::Result<()> {
        let sql = format!("DELETE FROM undo_log WHERE branch_id = {}", branch_id.0);
        self.db.execute_unprepared(&sql).await
            .map_err(|e| anyhow::anyhow!("Failed to delete undo logs: {}", e))?;
        Ok(())
    }

    async fn batch_delete_undo_logs(&self, branch_ids: Vec<BranchId>) -> anyhow::Result<()> {
        if branch_ids.is_empty() {
            return Ok(());
        }

        let ids: Vec<String> = branch_ids.iter().map(|id| id.0.to_string()).collect();
        let ids_str = ids.join(",");

        let sql = format!("DELETE FROM undo_log WHERE branch_id IN ({})", ids_str);

        self.db.execute_unprepared(&sql).await
            .map_err(|e| anyhow::anyhow!("Failed to batch delete undo logs: {}", e))?;
        Ok(())
    }
}

// Helper functions for creating undo logs
pub fn create_undo_log(
    branch_id: BranchId,
    xid: Xid,
    table_name: String,
    sql_type: SQLType,
    before_image: Option<RowImage>,
    after_image: Option<RowImage>,
) -> UndoLog {
    let now = chrono::Utc::now().timestamp_millis();
    UndoLog {
        branch_id,
        xid,
        table_name,
        sql_type,
        before_image,
        after_image,
        log_created: now,
        log_modified: now,
    }
}