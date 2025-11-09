use crate::branch::{BranchId, BranchStatus, BranchType};
use crate::types::{GlobalStatus, ResourceId, Xid};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionEventType {
    // 全局事务事件
    GlobalBegin {
        xid: Xid,
        timeout_millis: u64,
    },
    GlobalCommit {
        status: GlobalStatus,
        commit_duration_millis: u64,
    },
    GlobalRollback {
        status: GlobalStatus,
        rollback_duration_millis: u64,
    },
    GlobalStatusChange {
        from: GlobalStatus,
        to: GlobalStatus,
    },

    // 分支事务事件
    BranchRegister {
        branch_id: BranchId,
        branch_type: BranchType,
        resource_id: ResourceId,
        lock_keys: String,
    },
    BranchCommit {
        branch_id: BranchId,
        status: BranchStatus,
    },
    BranchRollback {
        branch_id: BranchId,
        status: BranchStatus,
    },
    BranchReport {
        branch_id: BranchId,
        status: BranchStatus,
    },

    // 资源管理事件
    ResourceRegistered {
        resource_id: ResourceId,
        branch_type: BranchType,
    },
    ResourceUnregistered {
        resource_id: ResourceId,
    },

    // 系统事件
    SessionTimeout {
        session_count: usize,
    },
    RecoveryStarted {
        recovered_sessions: usize,
    },
    RecoveryCompleted {
        total_sessions: usize,
    },
}
