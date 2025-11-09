use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

pub mod branch_manager_inbound;
pub mod branch_manager_outbound;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct BranchId(pub u64);
impl From<u64> for BranchId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl Into<u64> for BranchId {
    fn into(self) -> u64 {
        self.0
    }
}

impl Display for BranchId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BranchType {
    AT = 1,
    TCC = 2,
    SAGA = 3,
    XA = 4,
}
impl From<i32> for BranchType {
    fn from(value: i32) -> Self {
        match value {
            1 => BranchType::AT,
            2 => BranchType::TCC,
            3 => BranchType::SAGA,
            4 => BranchType::XA,
            _ => BranchType::AT,
        }
    }
}

impl Into<i32> for BranchType {
    fn into(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BranchStatus {
    Registered = 1,
    PhaseOneDone = 2,
    PhaseOneFailed = 3,
    PhaseOneTimeout = 4,
    PhaseTwoCommitted = 5,
    PhaseTwoCommitFailedRetryable = 6,
    PhaseTwoCommitFailedUnretryable = 7,
    PhaseTwoRollbacked = 8,
    PhaseTwoRollbackFailedRetryable = 9,
    PhaseTwoRollbackFailedUnretryable = 10,
    PhaseTwoTimeout = 11,
    Unknown = 12,
}
impl From<i32> for BranchStatus {
    fn from(value: i32) -> Self {
        match value {
            1 => Self::Registered,
            2 => Self::PhaseOneDone,
            3 => Self::PhaseOneFailed,
            4 => Self::PhaseOneTimeout,
            5 => Self::PhaseTwoCommitted,
            6 => Self::PhaseTwoCommitFailedRetryable,
            7 => Self::PhaseTwoCommitFailedUnretryable,
            8 => Self::PhaseTwoRollbacked,
            9 => Self::PhaseTwoRollbackFailedRetryable,
            10 => Self::PhaseTwoRollbackFailedUnretryable,
            11 => Self::PhaseTwoTimeout,
            _ => Self::Unknown,
        }
    }
}
impl Into<i32> for BranchStatus {
    fn into(self) -> i32 {
        self as i32
    }
}
