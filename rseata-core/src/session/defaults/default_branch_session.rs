use crate::branch::{BranchId, BranchStatus, BranchType};
use crate::error::TransactionError;
use crate::lock::LockStatus;
use crate::lock::lockable::Lockable;
use crate::session::branch_session::BranchSession;
use crate::session::session_storable::SessionStorable;
use crate::types::{ClientId, ResourceId, Xid};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultBranchSession {
    pub xid: Xid,
    pub transaction_id: u64,
    pub branch_id: BranchId,
    pub resource_group_id: Option<String>,
    pub resource_id: Option<ResourceId>,
    pub lock_key: Option<String>,
    pub branch_type: BranchType,
    pub status: BranchStatus,
    pub client_id: ClientId,
    pub application_data: Option<String>,
    pub lock_status: LockStatus,
    pub lock_holder: HashMap<String, Vec<String>>,
}

impl DefaultBranchSession {
    pub fn new(branch_type: BranchType) -> Self {
        Self {
            xid: Xid::from(Uuid::new_v4().to_string()),
            transaction_id: 0,
            branch_id: 0.into(),
            resource_group_id: None,
            resource_id: None,
            lock_key: None,
            branch_type,
            status: BranchStatus::Unknown,
            client_id: 0.into(),
            application_data: None,
            lock_status: LockStatus::Locked,
            lock_holder: HashMap::new(),
        }
    }
}

impl BranchSession for DefaultBranchSession {
    fn xid(&self) -> &Xid {
        &self.xid
    }

    fn transaction_id(&self) -> u64 {
        self.transaction_id
    }

    fn branch_id(&self) -> BranchId {
        self.branch_id
    }

    fn resource_group_id(&self) -> Option<&str> {
        self.resource_group_id.as_deref()
    }

    fn resource_id(&self) -> &Option<ResourceId> {
        &self.resource_id
    }

    fn lock_key(&self) -> Option<&str> {
        self.lock_key.as_deref()
    }

    fn branch_type(&self) -> BranchType {
        self.branch_type.clone()
    }

    fn status(&self) -> BranchStatus {
        self.status.clone()
    }

    fn client_id(&self) -> ClientId {
        self.client_id
    }

    fn application_data(&self) -> Option<&str> {
        self.application_data.as_deref()
    }

    fn lock_status(&self) -> LockStatus {
        self.lock_status.clone()
    }
}

#[async_trait]
impl Lockable for DefaultBranchSession {
    async fn lock(&self) -> Result<bool, TransactionError> {
        self.lock_internal(true, false).await
    }

    async fn unlock(&self) -> Result<bool, TransactionError> {
        if self.branch_type == BranchType::AT {
            // Use lock manager to release lock
            // lock_manager.release_lock(self).await
            Ok(true) // Placeholder
        } else {
            Ok(true)
        }
    }
}

impl DefaultBranchSession {
    async fn lock_internal(
        &self,
        _auto_commit: bool,
        _skip_check_lock: bool,
    ) -> Result<bool, TransactionError> {
        if self.branch_type == BranchType::AT {
            // Use lock manager to acquire lock
            // lock_manager.acquire_lock(self, auto_commit, skip_check_lock).await
            Ok(true) // Placeholder
        } else {
            Ok(true)
        }
    }
    pub(crate) fn can_be_committed_async(&self) -> bool {
        true
    }
}

impl SessionStorable for DefaultBranchSession {
    fn encode(&self) -> Result<Vec<u8>, TransactionError> {
        // Implementation for encoding branch session to bytes
        // Similar to Java version's encode method
        Ok(serde_json::to_vec(self).map_err(|e| TransactionError::new(e.to_string()))?)
    }

    fn decode(data: &[u8]) -> Result<Self, TransactionError> {
        serde_json::from_slice(data).map_err(|e| TransactionError::new(e.to_string()))
    }

    fn max_size(&self) -> usize {
        todo!()
    }
}

impl std::fmt::Display for DefaultBranchSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BR:{}/{}", self.branch_id, self.transaction_id)
    }
}

impl PartialEq for DefaultBranchSession {
    fn eq(&self, other: &Self) -> bool {
        self.branch_id == other.branch_id
    }
}

impl Eq for DefaultBranchSession {}

impl PartialOrd for DefaultBranchSession {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DefaultBranchSession {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.branch_id.0.cmp(&other.branch_id.0)
    }
}
