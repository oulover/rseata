#[cfg(feature = "tm")]
pub use rseata_tm::FutureExt;
#[cfg(feature = "tm")]
pub use rseata_tm::RSEATA_CLIENT_SESSION;
#[cfg(feature = "tm")]
pub use rseata_tm::RSEATA_TM;
#[cfg(feature = "tm")]
pub use rseata_tm::RseataTM;

#[cfg(feature = "rm")]
pub use rseata_rm::RSEATA_RM;

#[cfg(feature = "micros")]
pub use rseata_micro::global_transaction;

pub mod core {
    pub use rseata_core::ClientSession;
    pub use rseata_core::grpc_layer;
    pub use rseata_core::grpc_server_interceptor::GrpcServerInterceptor;
    pub use rseata_core::transaction::transaction_manager::TransactionManager;
}

pub mod db_proxy {

    #[cfg(feature = "sea_orm")]
    pub mod sea_orm {
        pub use rseata_db_proxy::sea_orm::at::connection_proxy::ATConnectionProxy;
        pub use rseata_db_proxy::sea_orm::at::transaction_proxy::ATTransactionProxy;

        pub use rseata_db_proxy::sea_orm::xa::connection_proxy::XAConnectionProxy;
        pub use rseata_db_proxy::sea_orm::xa::transaction_proxy::XATransactionProxy;
    }

    #[cfg(feature = "diesel")]
    pub mod diesel {}
}

pub async fn init() {

    #[cfg(feature = "rm")]
    rseata_rm::init().await;
}
