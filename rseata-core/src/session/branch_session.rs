use crate::branch::{BranchId, BranchStatus, BranchType};
use crate::lock::LockStatus;
use crate::lock::lockable::Lockable;
use crate::session::session_storable::SessionStorable;
use crate::types::{ClientId, ResourceId, Xid};

pub trait BranchSession: Lockable + SessionStorable + Send + Sync {
    fn xid(&self) -> &Xid;
    fn transaction_id(&self) -> u64;
    fn branch_id(&self) -> BranchId;
    fn resource_group_id(&self) -> Option<&str>;
    fn resource_id(&self) -> &Option<ResourceId>;
    fn lock_key(&self) -> Option<&str>;
    fn branch_type(&self) -> BranchType;
    fn status(&self) -> BranchStatus;
    fn client_id(&self) -> ClientId;
    fn application_data(&self) -> Option<&str>;
    fn lock_status(&self) -> LockStatus;
}
