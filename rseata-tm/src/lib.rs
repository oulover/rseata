pub mod transaction_manager;

pub static RSEATA_TM: RseataTM = RseataTM;
pub use futures::FutureExt;
pub use rseata_core::RSEATA_CLIENT_SESSION;

pub use rseata_core::transaction::transaction_manager::TransactionManager;
pub use transaction_manager::RseataTM;


pub async fn init(){
    dotenv::dotenv().ok();

}