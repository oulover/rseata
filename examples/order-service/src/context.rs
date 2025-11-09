
use std::sync::Arc;
use example_proto::client::LazyTestUserGrpcClient;
use rseata::db_proxy::sea_orm::ATConnectionProxy;

pub struct AppContext {
    pub db_conn: ATConnectionProxy,
    pub user_client: Arc<LazyTestUserGrpcClient>,
}
