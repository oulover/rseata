use rseata::db_proxy::sea_orm::ATConnectionProxy;
#[derive(Debug)]
pub struct AppContext {
    pub db_conn:ATConnectionProxy,
}