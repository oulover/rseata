use crate::session::branch_session::BranchSession;
use crate::session::session_storable::SessionStorable;
use crate::types::{GlobalStatus, Xid};
pub trait GlobalSession: SessionStorable + Send + Sync {
    type BranchSession: BranchSession + Send + Sync;
    fn xid(&self) -> &Xid;
    fn transaction_id(&self) -> u64;
    fn status(&self) -> GlobalStatus;
    fn application_id(&self) -> &str;
    fn transaction_service_group(&self) -> &str;
    fn transaction_name(&self) -> &str;
    fn timeout_millis(&self) -> u64;
    fn begin_time_millis(&self) -> u64;
    fn application_data(&self) -> Option<&str>;
    fn lazy_load_branch(&self) -> bool;
    fn active(&self) -> bool;
    fn branch_sessions(&self) -> Vec<Self::BranchSession>;
}
