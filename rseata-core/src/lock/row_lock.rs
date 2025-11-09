use crate::branch::BranchId;
use crate::types::{ResourceId, Xid};

pub trait RowLock:From<RowLockData> {
    fn xid(&self) -> &Xid;
    fn transaction_id(&self) -> u64;
    fn branch_id(&self) -> Option<BranchId>;
    fn resource_id(&self) -> &ResourceId;
    fn table_name(&self) -> &str;
    fn pk(&self) -> &str;
    fn row_key(&self) -> Option<&str>;
    fn feature(&self) -> Option<&str>;
}
pub struct RowLockData {
    pub xid: Xid,
    pub transaction_id: u64,
    pub branch_id: Option<BranchId>,
    pub resource_id: ResourceId,
    pub table_name: String,
    pub pk: String,
    pub row_key: Option<String>,
    pub feature: Option<String>,
}
