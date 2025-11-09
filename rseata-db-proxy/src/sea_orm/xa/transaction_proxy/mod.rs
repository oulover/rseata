mod impl_connection_trait;
mod impl_stream_trait;
mod impl_transaction_session;
mod impl_transaction_trait;

use crate::sea_orm::xa::connection_proxy::XAConnectionProxy;
use rseata_core::branch::branch_manager_outbound::BranchManagerOutbound;
use rseata_core::branch::BranchType;
use rseata_core::resource::Resource;
use rseata_core::RSEATA_CLIENT_SESSION;
use rseata_rm::RSEATA_RM;
use sea_orm::{ConnectionTrait, DatabaseTransaction, DbErr};

pub enum TransactionType {
    Local(DatabaseTransaction),
    XA(String),
}

pub struct XATransactionProxy {
    pub transaction_type: TransactionType,
    pub xa_connection_proxy: XAConnectionProxy,
}

impl XATransactionProxy {
    // xa start in connection
    async fn xa_end(&self, xa_id: &str) -> Result<sea_orm::ExecResult, DbErr> {
        let sql = format!("XA END '{xa_id}'");
        self.execute_unprepared(&sql).await
    }
    async fn xa_prepare(&self, xa_id: &str) -> Result<sea_orm::ExecResult, DbErr> {
        let sql = format!("XA PREPARE '{xa_id}'");
        self.execute_unprepared(&sql).await
    }
    async fn xa_commit(&self, xa_id: &str) -> Result<sea_orm::ExecResult, DbErr> {
        let sql = format!("XA COMMIT '{xa_id}'");
        self.execute_unprepared(&sql).await
    }

    async fn xa_rollback(&self, xa_id: &str) -> Result<sea_orm::ExecResult, DbErr> {
        let sql = format!("XA ROLLBACK '{xa_id}'");
        self.execute_unprepared(&sql).await
    }
}

impl std::fmt::Debug for XATransactionProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DatabaseTransaction")
    }
}

impl XATransactionProxy {
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

    pub async fn global_commit(
        local_commit_result: Result<sea_orm::ExecResult, DbErr>,
    ) -> Result<sea_orm::ExecResult, DbErr> {
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
