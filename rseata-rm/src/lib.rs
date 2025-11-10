use crate::resource::{DefaultResourceManager, ResourceInfo};
use lazy_static::lazy_static;
use rseata_core::branch::BranchType;

mod config;
pub mod resource;

lazy_static! {
    pub static ref RSEATA_RM: DefaultResourceManager =
        DefaultResourceManager::new(ResourceInfo::new_with_env(BranchType::AT));
}

pub async fn init() {
    tracing::info!("RSEATA_RM init....");
    RSEATA_RM.init().await;
    tracing::info!("RSEATA_RM init end");
}
