pub mod transaction_manager;

pub use futures::FutureExt;
use lazy_static::lazy_static;
pub use rseata_core::RSEATA_CLIENT_SESSION;

pub use rseata_core::transaction::transaction_manager::TransactionManager;
pub use transaction_manager::RseataTM;
lazy_static! {
    pub static ref RSEATA_TM: RseataTM = RseataTM::new_with_env();
}

pub async fn init() {
    dotenv::dotenv().ok();
}
