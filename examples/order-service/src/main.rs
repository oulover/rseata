use crate::context::AppContext;
use example_proto::client::{GrpcCtx, LazyTestUserGrpcClient};
use std::env;
use std::sync::Arc;

mod context;
mod order;
mod web;
mod service;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    rseata::init().await;


    let connect_url = dotenv::var("ORDER_DATABASE_URL")
        .unwrap_or("mysql://root:root@127.0.0.1:3306/order".to_string());

    let conn = rseata::db_proxy::sea_orm::XAConnectionProxy::connect_mysql(&connect_url).await?;

    let user_endpoint =
        dotenv::var("USER_ENDPOINT").unwrap_or(String::from("http://127.0.0.1:9001"));

    let app_ctx = Arc::new(AppContext {
        db_conn: conn,
        user_client: Arc::new(LazyTestUserGrpcClient::new(GrpcCtx {
            endpoint: user_endpoint,
        })),
    });

    web::start(app_ctx).await?;
    Ok(())
}
