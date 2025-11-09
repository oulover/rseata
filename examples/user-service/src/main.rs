use crate::context::AppContext;
use std::sync::Arc;

mod context;
mod grpc_service;
mod user;
mod web;
pub mod service;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    rseata::init().await;

    let connect_url = dotenv::var("USER_DATABASE_URL")
        .unwrap_or("mysql://root:root@127.0.0.1:3306/user".to_string());

    let conn = rseata::db_proxy::sea_orm::ATConnectionProxy::connect_mysql(&connect_url).await?;
    let app_ctx = Arc::new(AppContext { db_conn: conn });

    tokio::spawn(grpc_service::start(app_ctx.clone()));
    web::start(app_ctx).await?;
    Ok(())
}
