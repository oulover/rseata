use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Xid(pub String);
impl From<&str> for Xid {
    fn from(value: &str) -> Self {
        Self(String::from(value))
    }
}
impl From<String> for Xid {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl Display for Xid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceId(pub String);
impl From<&str> for ResourceId {
    fn from(value: &str) -> Self {
        Self(String::from(value))
    }
}
impl From<String> for ResourceId {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl Display for ResourceId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum GlobalStatus {
    UnKnown = 0,
    Begin = 1,
    Committing = 2,
    CommitRetrying = 3,
    Rollbacking = 4,
    RollbackRetrying = 5,
    TimeoutRollbacking = 6,
    TimeoutRollbackRetrying = 7,
    AsyncCommitting = 8,
    Committed = 9,
    CommitFailed = 10,
    Rollbacked = 11,
    RollbackFailed = 12,
    TimeoutRollbacked = 13,
    TimeoutRollbackFailed = 14,
    Finished = 15,
    CommitRetryTimeout = 16,
    RollbackRetryTimeout = 17,
    Deleting = 18,
    StopCommitOrCommitRetry = 19,
    StopRollbackOrRollbackRetry = 20,
}

impl GlobalStatus {
    pub fn code(&self) -> i32 {
        match self {
            Self::UnKnown => 0,
            Self::Begin => 1,
            Self::Committing => 2,
            Self::CommitRetrying => 3,
            Self::Rollbacking => 4,
            Self::RollbackRetrying => 5,
            Self::TimeoutRollbacking => 6,
            Self::TimeoutRollbackRetrying => 7,
            Self::AsyncCommitting => 8,
            Self::Committed => 9,
            Self::CommitFailed => 10,
            Self::Rollbacked => 11,
            Self::RollbackFailed => 12,
            Self::TimeoutRollbacked => 13,
            Self::TimeoutRollbackFailed => 14,
            Self::Finished => 15,
            Self::CommitRetryTimeout => 16,
            Self::RollbackRetryTimeout => 17,
            Self::Deleting => 18,
            Self::StopCommitOrCommitRetry => 19,
            Self::StopRollbackOrRollbackRetry => 20,
        }
    }

    pub fn desc(&self) -> &'static str {
        match self {
            Self::UnKnown => "an ambiguous transaction state, usually use before begin",
            Self::Begin => "global transaction start",
            Self::Committing => "2Phase committing",
            Self::CommitRetrying => "2Phase committing failure retry",
            Self::Rollbacking => "2Phase rollbacking",
            Self::RollbackRetrying => "2Phase rollbacking failure retry",
            Self::TimeoutRollbacking => "after global transaction timeout rollbacking",
            Self::TimeoutRollbackRetrying => "after global transaction timeout rollback retrying",
            Self::AsyncCommitting => "2Phase committing, used for AT mode",
            Self::Committed => "global transaction completed with status committed",
            Self::CommitFailed => "2Phase commit failed",
            Self::Rollbacked => "global transaction completed with status rollbacked",
            Self::RollbackFailed => "global transaction completed but rollback failed",
            Self::TimeoutRollbacked => "global transaction completed with rollback due to timeout",
            Self::TimeoutRollbackFailed => {
                "global transaction was rollbacking due to timeout, but failed"
            }
            Self::Finished => {
                "ambiguous transaction status for non-exist transaction and global report for Saga"
            }
            Self::CommitRetryTimeout => {
                "global transaction still failed after commit failure and retries for some time"
            }
            Self::RollbackRetryTimeout => {
                "global transaction still failed after commit failure and retries for some time"
            }
            Self::Deleting => "global transaction is deleting",
            Self::StopCommitOrCommitRetry => {
                "global transaction is commit or retry commit but stop now"
            }
            Self::StopRollbackOrRollbackRetry => {
                "global transaction is rollback or retry rollback but stop now"
            }
        }
    }

    pub fn from_code(code: i32) -> Result<Self, String> {
        match code {
            0 => Ok(Self::UnKnown),
            1 => Ok(Self::Begin),
            2 => Ok(Self::Committing),
            3 => Ok(Self::CommitRetrying),
            4 => Ok(Self::Rollbacking),
            5 => Ok(Self::RollbackRetrying),
            6 => Ok(Self::TimeoutRollbacking),
            7 => Ok(Self::TimeoutRollbackRetrying),
            8 => Ok(Self::AsyncCommitting),
            9 => Ok(Self::Committed),
            10 => Ok(Self::CommitFailed),
            11 => Ok(Self::Rollbacked),
            12 => Ok(Self::RollbackFailed),
            13 => Ok(Self::TimeoutRollbacked),
            14 => Ok(Self::TimeoutRollbackFailed),
            15 => Ok(Self::Finished),
            16 => Ok(Self::CommitRetryTimeout),
            17 => Ok(Self::RollbackRetryTimeout),
            18 => Ok(Self::Deleting),
            19 => Ok(Self::StopCommitOrCommitRetry),
            20 => Ok(Self::StopRollbackOrRollbackRetry),
            _ => Err(format!("Unknown GlobalStatus[{}]", code)),
        }
    }

    pub fn is_one_phase_timeout(status: Self) -> bool {
        matches!(
            status,
            Self::TimeoutRollbacking
                | Self::TimeoutRollbackRetrying
                | Self::TimeoutRollbacked
                | Self::TimeoutRollbackFailed
        )
    }

    pub fn is_two_phase_success(status: Self) -> bool {
        matches!(
            status,
            Self::Committed | Self::Rollbacked | Self::TimeoutRollbacked | Self::Deleting
        )
    }

    pub fn is_two_phase_heuristic(status: Self) -> bool {
        matches!(status, Self::Finished)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ClientId(pub u64);
impl From<u64> for ClientId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl Into<u64> for ClientId {
    fn into(self) -> u64 {
        self.0
    }
}

impl Display for ClientId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
