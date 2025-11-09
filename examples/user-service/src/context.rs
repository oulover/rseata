use rseata::db_proxy::sea_orm::ConnectionProxy;
#[derive(Debug)]
pub struct AppContext {
    pub db_conn:ConnectionProxy,
}