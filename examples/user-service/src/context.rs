use rseata::db_proxy::sea_orm::XAConnectionProxy;
#[derive(Debug)]
pub struct AppContext {
    pub db_conn:XAConnectionProxy,
}