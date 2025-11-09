use crate::branch::BranchType;

pub trait HandleBranchType {
    fn handle_branch_type(&self) -> BranchType;
}