
use std::sync::Arc;
use example_proto::client::LazyTestUserGrpcClient;
use rseata::db_proxy::sea_orm::ConnectionProxy;

pub struct AppContext {
    pub db_conn: ConnectionProxy,
    pub user_client: Arc<LazyTestUserGrpcClient>,
}
