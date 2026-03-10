use crate::sea_orm::xa::transaction_proxy::{TransactionType, XATransactionProxy};
use rseata_core::RSEATA_CLIENT_SESSION;
use rseata_core::branch::BranchType;
use rseata_core::branch::branch_manager_outbound::BranchManagerOutbound;
use rseata_core::resource::Resource;
use rseata_core::types::Xid;
use rseata_rm::RSEATA_RM;
use rseata_tm::RSEATA_TM;
use sea_orm::{DbErr, TransactionSession};

#[async_trait::async_trait]
impl TransactionSession for XATransactionProxy {
    async fn commit(self) -> Result<(), DbErr> {
        let session = RSEATA_CLIENT_SESSION.try_get().ok();

        match self.transaction_type {
            TransactionType::Local(local) => {
                let r = local.lock().await.take();
                if let Some(r) = r {
                    r.commit().await
                } else {
                    Ok(())
                }
            }
            TransactionType::XA(ref xa_transaction) => {
                // 1. 注册分支事务
                self.branch_register().await?;

                // 2. 执行 XA END
                let end_result = xa_transaction.xa_end().await;

                // 3. 检查全局锁
                let locked = self.check_lock().await?;
                if !locked {
                    tracing::error!("Check lock failed");
                    return self.rollback().await;
                }

                match end_result {
                    Ok(_) => {
                        // 4. 执行 XA PREPARE
                        let prepare_result = xa_transaction.xa_prepare().await;

                        match prepare_result {
                            Ok(_) => {
                                // 5. 报告 PhaseOneDone 状态
                                if let Some(session) = session {
                                    if session.is_global_tx_started() {
                                        if let Some(xid) = session.get_xid() {
                                            RSEATA_RM
                                                .branch_report(
                                                    BranchType::XA,
                                                    xid,
                                                    session.get_branch_id(),
                                                    rseata_core::branch::BranchStatus::PhaseOneDone,
                                                    String::from(""),
                                                )
                                                .await
                                                .map_err(|e| DbErr::Custom(e.to_string()))?;
                                        }
                                    }
                                }

                                // 6. 执行 XA COMMIT
                                let commit_result = xa_transaction.xa_commit().await;

                                match commit_result {
                                    Ok(_) => XATransactionProxy::report_local_commit(Ok(())).await,
                                    Err(e) => {
                                        // 如果 xa_commit 失败，也要报告失败
                                        XATransactionProxy::report_local_commit(Err(e)).await
                                    }
                                }
                            }
                            Err(e) => {
                                // PREPARE 失败，执行回滚
                                let _ = self.rollback().await;
                                Err(e)
                            }
                        }
                    }
                    Err(e) => {
                        // END 失败，执行回滚
                        let _ = self.rollback().await;
                        Err(e)
                    }
                }
            }
        }
    }

    async fn rollback(self) -> Result<(), DbErr> {
        match &self.transaction_type {
            TransactionType::Local(local) => {
                let r = local.lock().await.take();
                if let Some(r) = r {
                    r.rollback().await
                } else {
                    Ok(())
                }
            }
            TransactionType::XA(xa_transaction) => {
                self.branch_register().await?;

                // 1. 尝试执行 XA END（如果还没有执行）
                let _ = xa_transaction.xa_end().await; // 忽略错误

                // 2. 执行 XA ROLLBACK
                let rollback_result = xa_transaction.xa_rollback().await;

                // 3. 报告回滚状态
                let _ = XATransactionProxy::report_local_rollback().await?;

                rollback_result.map(|_| ())
            }
        }
    }
}
