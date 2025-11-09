use crate::branch::BranchId;
use crate::lock::row_lock::{RowLock, RowLockData};
use crate::types::{ResourceId, Xid};

#[derive(Debug, Clone)]
pub struct DefaultRowLock {
    pub xid: Xid,
    pub transaction_id: u64,
    pub branch_id: Option<BranchId>,
    pub resource_id: ResourceId,
    pub table_name: String,
    pub pk: String,
    pub row_key: Option<String>,
    pub feature: Option<String>,
}

impl From<RowLockData> for DefaultRowLock {
    fn from(value: RowLockData) -> Self {
        Self {
            xid: value.xid,
            transaction_id: value.transaction_id,
            branch_id: value.branch_id,
            resource_id: value.resource_id,
            table_name: value.table_name,
            pk: value.pk,
            row_key: value.row_key,
            feature: value.feature,
        }
    }
}

impl RowLock for DefaultRowLock {
    fn xid(&self) -> &Xid {
        &self.xid
    }

    fn transaction_id(&self) -> u64 {
        self.transaction_id
    }

    fn branch_id(&self) -> Option<BranchId> {
        self.branch_id
    }

    fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }

    fn table_name(&self) -> &str {
        &self.table_name
    }

    fn pk(&self) -> &str {
        &self.pk
    }

    fn row_key(&self) -> Option<&str> {
        self.row_key.as_deref()
    }

    fn feature(&self) -> Option<&str> {
        self.feature.as_deref()
    }
}

impl DefaultRowLock {
    pub fn new(
        xid: Xid,
        transaction_id: u64,
        resource_id: ResourceId,
        branch_id: Option<BranchId>,
        table_name: String,
        pk: String,
    ) -> Self {
        Self {
            xid,
            transaction_id,
            branch_id,
            resource_id,
            table_name,
            pk,
            row_key: None,
            feature: None,
        }
    }
}
