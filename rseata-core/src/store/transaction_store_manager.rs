use std::fmt::Debug;

use crate::session::global_session::GlobalSession;
use crate::session::session_condition::SessionCondition;
use crate::store::LogOperation;
use crate::types::{GlobalStatus, Xid};
use async_trait::async_trait;

#[async_trait]
pub trait TransactionStoreManager: Send + Sync + Debug {
    type GlobalSession: GlobalSession + Send + Sync;

    /// Write session with specified operation
    async fn write_session(
        &self,
        log_operation: LogOperation,
        session: &Self::GlobalSession,
    ) -> anyhow::Result<()>;

    async fn read_session(&self, xid: &Xid) -> Option<Self::GlobalSession>;

    async fn read_session_with_branches(&self, xid: &Xid) -> Option<Self::GlobalSession>;

    /// Read global session by xid
    async fn read_global_session(
        &self,
        xid: &Xid,
        with_branch_sessions: bool,
    ) -> Option<Self::GlobalSession>;

    async fn read_sort_by_timeout_begin_sessions(
        &self,
        with_branch_sessions: bool,
    ) -> Vec<Self::GlobalSession>;

    async fn read_session_by_global_status(
        &self,
        statuses: &Vec<GlobalStatus>,
        with_branch_sessions: bool,
    ) -> Vec<Self::GlobalSession>;

    async fn read_session_by_session_condition(
        &self,
        statuses: &SessionCondition,
    ) -> Vec<Self::GlobalSession>;
}
