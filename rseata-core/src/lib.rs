pub use crate::session::ClientSession;
use std::sync::Arc;
use tokio::task_local;

pub mod branch;
pub mod coordinator;
pub mod error;
pub mod event;
pub mod grpc_client;
pub mod grpc_layer;
pub mod grpc_server_interceptor;
pub mod handle_branch_type;
pub mod lock;
pub mod resource;
pub mod session;
pub mod store;
pub mod transaction;
pub mod types;

pub static EMPTY_STR: &'static str = "";
pub static RSEATA_VERSION: &'static str = "0.0.1";

pub static RSEATA_XID_KEY: &'static str = "RSEATA_XID";

task_local! {
   pub static RSEATA_CLIENT_SESSION: Arc<ClientSession>;
}
