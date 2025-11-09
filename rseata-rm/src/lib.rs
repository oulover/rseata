use crate::resource::{ResourceInfo, ResourceService};
use lazy_static::lazy_static;
use rseata_core::branch::BranchType;

mod config;
pub mod resource;

lazy_static! {
    pub static ref RSEATA_RM: ResourceService = ResourceService::new(ResourceInfo::new(
        String::from("resource_group_id--2"),
        BranchType::AT
    ));
}

pub async fn init() {
    println!("RSEATA_RM init....");
    RSEATA_RM.init().await;
    println!("RSEATA_RM init ended!");
}
