use crate::context::AppContext;
use std::sync::Arc;
use rseata::db_proxy::sea_orm::ConnectionProxy;

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

    let connect_url = dotenv::var("DEMO_1_DATABASE_URL")
        .unwrap_or("mysql://root:root@127.0.0.1:3306/user".to_string());

    let conn = ConnectionProxy::connect(&connect_url).await?;
    let app_ctx = Arc::new(AppContext { db_conn: conn });

    tokio::spawn(grpc_service::start(app_ctx.clone()));
    web::start(app_ctx).await?;
    Ok(())
}
