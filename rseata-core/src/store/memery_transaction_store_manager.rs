use crate::session::defaults::default_global_session::DefaultGlobalSession;
use crate::session::session_condition::SessionCondition;
use crate::session::session_storable::SessionStorable;
use crate::store::LogOperation;
use crate::store::transaction_store_manager::TransactionStoreManager;
use crate::types::{GlobalStatus, Xid};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
struct MemorySessionStore {
    // 按 XID 索引的全局会话
    global_sessions: HashMap<Xid, DefaultGlobalSession>,
    // 按开始时间排序的会话索引，用于超时查询
    timeout_index: BTreeMap<DateTime<Utc>, Xid>,
    // 按状态索引的会话
    status_index: HashMap<GlobalStatus, Vec<Xid>>,
}

impl MemorySessionStore {
    fn new() -> Self {
        Self {
            global_sessions: HashMap::new(),
            timeout_index: BTreeMap::new(),
            status_index: HashMap::new(),
        }
    }

    fn add_global_session(&mut self, session: DefaultGlobalSession) {
        let xid = session.xid.clone();
        let begin_time = DateTime::from_timestamp_millis(session.begin_time_millis as i64)
            .unwrap_or_else(Utc::now);
        let status = session.status;

        // 插入主存储
        self.global_sessions.insert(xid.clone(), session);

        // 更新超时索引
        self.timeout_index.insert(begin_time, xid.clone());

        // 更新状态索引
        self.status_index
            .entry(status)
            .or_insert_with(Vec::new)
            .push(xid);
    }

    fn update_global_session_status(&mut self, xid: &Xid, new_status: GlobalStatus) -> bool {
        if let Some(session) = self.global_sessions.get_mut(xid) {
            let old_status = session.status;

            // 如果状态改变，更新索引
            if old_status != new_status {
                // 从旧状态索引中移除
                if let Some(ids) = self.status_index.get_mut(&old_status) {
                    ids.retain(|id| id != xid);
                    if ids.is_empty() {
                        self.status_index.remove(&old_status);
                    }
                }

                // 添加到新状态索引
                self.status_index
                    .entry(new_status)
                    .or_insert_with(Vec::new)
                    .push(xid.clone());

                session.status = new_status;
                return true;
            }
        }
        false
    }

    fn remove_global_session(&mut self, xid: &Xid) -> Option<DefaultGlobalSession> {
        if let Some(session) = self.global_sessions.remove(xid) {
            let begin_time = DateTime::from_timestamp_millis(session.begin_time_millis as i64)
                .unwrap_or_else(Utc::now);

            // 从超时索引中移除
            self.timeout_index.retain(|_, id| id != xid);

            // 从状态索引中移除
            if let Some(ids) = self.status_index.get_mut(&session.status) {
                ids.retain(|id| id != xid);
                if ids.is_empty() {
                    self.status_index.remove(&session.status);
                }
            }

            Some(session)
        } else {
            None
        }
    }

    fn get_global_session(&self, xid: &Xid) -> Option<DefaultGlobalSession> {
        self.global_sessions.get(xid).cloned()
    }

    fn get_sessions_by_status(&self, status: &GlobalStatus) -> Vec<DefaultGlobalSession> {
        self.status_index
            .get(status)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.global_sessions.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn get_sessions_sorted_by_timeout(&self) -> Vec<DefaultGlobalSession> {
        self.timeout_index
            .values()
            .filter_map(|xid| self.global_sessions.get(xid).cloned())
            .collect()
    }

    fn get_sessions_by_condition(&self, condition: &SessionCondition) -> Vec<DefaultGlobalSession> {
        // 这里需要根据 SessionCondition 的具体结构来实现过滤逻辑
        // 暂时返回所有会话，实际实现中需要根据条件过滤
        self.global_sessions.values().cloned().collect()
    }
}

#[derive(Debug)]
pub struct MemeryTransactionStoreManager {
    store: Arc<RwLock<MemorySessionStore>>,
}

impl Default for MemeryTransactionStoreManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MemeryTransactionStoreManager {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(MemorySessionStore::new())),
        }
    }

    async fn apply_log_operation(
        &self,
        log_operation: LogOperation,
        session: &dyn SessionStorable,
    ) -> anyhow::Result<()> {
        let mut store = self.store.write().await;

        match log_operation {
            LogOperation::GlobalAdd | LogOperation::GlobalUpdate => {
                if let Ok(global_session) = DefaultGlobalSession::decode(&session.encode()?) {
                    store.add_global_session(global_session);
                }
            }
            LogOperation::GlobalRemove => {
                if let Ok(global_session) = DefaultGlobalSession::decode(&session.encode()?) {
                    store.remove_global_session(&global_session.xid);
                }
            }
            LogOperation::BranchAdd | LogOperation::BranchUpdate | LogOperation::BranchRemove => {
                if let Ok(global_session) = DefaultGlobalSession::decode(&session.encode()?) {
                    store.add_global_session(global_session);
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl TransactionStoreManager for MemeryTransactionStoreManager {
    type GlobalSession = DefaultGlobalSession;

    async fn write_session(
        &self,
        log_operation: LogOperation,
        session: &Self::GlobalSession,
    ) -> anyhow::Result<()> {
        self.apply_log_operation(log_operation, session).await
    }

    async fn read_session(&self, xid: &Xid) -> Option<DefaultGlobalSession> {
        let store = self.store.read().await;
        store.get_global_session(xid)
    }

    async fn read_session_with_branches(&self, xid: &Xid) -> Option<DefaultGlobalSession> {
        // 在内存存储中，会话总是包含分支信息
        self.read_session(xid).await
    }

    async fn read_global_session(
        &self,
        xid: &Xid,
        with_branch_sessions: bool,
    ) -> Option<DefaultGlobalSession> {
        // 内存存储中分支会话信息总是可用的
        let session = self.read_session(xid).await;
        if with_branch_sessions {
            session
        } else {
            // 如果不要求分支会话，返回一个不包含分支会话的副本
            session.map(|mut s| {
                // 这里需要根据 DefaultGlobalSession 的实际结构来清空分支会话
                // 暂时返回原会话
                s
            })
        }
    }

    async fn read_sort_by_timeout_begin_sessions(
        &self,
        with_branch_sessions: bool,
    ) -> Vec<DefaultGlobalSession> {
        let store = self.store.read().await;
        let mut sessions = store.get_sessions_sorted_by_timeout();

        if !with_branch_sessions {
            // 如果不要求分支会话，清空所有会话的分支信息
            for session in &mut sessions {
                // 这里需要根据 DefaultGlobalSession 的实际结构来清空分支会话
                // 暂时保持原样
            }
        }

        sessions
    }

    async fn read_session_by_global_status(
        &self,
        statuses: &Vec<GlobalStatus>,
        with_branch_sessions: bool,
    ) -> Vec<DefaultGlobalSession> {
        let store = self.store.read().await;
        let mut result = Vec::new();

        for status in statuses {
            let mut sessions = store.get_sessions_by_status(status);
            if !with_branch_sessions {
                // 清空分支会话信息
                for session in &mut sessions {
                    // 这里需要根据 DefaultGlobalSession 的实际结构来清空分支会话
                }
            }
            result.extend(sessions);
        }

        result
    }

    async fn read_session_by_session_condition(
        &self,
        condition: &SessionCondition,
    ) -> Vec<DefaultGlobalSession> {
        let store = self.store.read().await;
        store.get_sessions_by_condition(condition)
    }
}
