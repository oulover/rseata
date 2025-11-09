use crate::types::{GlobalStatus, Xid};

#[derive(Debug, Clone)]
pub struct SessionCondition {
    pub transaction_id: Option<u64>,
    pub xid: Option<Xid>,
    pub status: Option<GlobalStatus>,
    pub statuses: Vec<GlobalStatus>,
    pub over_time_alive_mills: Option<u64>,
    pub lazy_load_branch: bool,
}

impl SessionCondition {
    pub fn new() -> Self {
        Self {
            transaction_id: None,
            xid: None,
            status: None,
            statuses: Vec::new(),
            over_time_alive_mills: None,
            lazy_load_branch: false,
        }
    }

    pub fn with_xid(xid: Xid) -> Self {
        Self {
            xid: Some(xid),
            ..Self::new()
        }
    }

    pub fn with_status(status: GlobalStatus) -> Self {
        Self {
            status: Some(status),
            statuses: vec![status],
            ..Self::new()
        }
    }

    pub fn with_statuses(statuses: Vec<GlobalStatus>) -> Self {
        Self {
            statuses,
            ..Self::new()
        }
    }

    pub fn with_over_time(over_time_alive_mills: u64) -> Self {
        Self {
            over_time_alive_mills: Some(over_time_alive_mills),
            ..Self::new()
        }
    }
}
