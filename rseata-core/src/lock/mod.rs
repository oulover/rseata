pub mod defaults;
pub mod lock_manager;
pub mod lockable;
mod locker;
pub mod row_lock;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LockStatus {
    Locked = 1,
    Rollbacking = 2,
}
