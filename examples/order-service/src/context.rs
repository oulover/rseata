
use std::sync::Arc;
use example_proto::client::LazyTestUserGrpcClient;
use rseata::db_proxy::sea_orm::XAConnectionProxy;

pub struct AppContext {
    pub db_conn: XAConnectionProxy,
    pub user_client: Arc<LazyTestUserGrpcClient>,
}
