use crate::config;
use crate::context::Context;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use rseata_core::event::event::TransactionEvent;
use std::sync::Arc;

pub(crate) async fn start(ctx: Arc<Context>) -> anyhow::Result<()> {
    let ip_prot = format!(
        "{}:{}",
        config::get_env_web_server_ip(),
        config::get_env_web_server_prot()
    );
    let app = Router::new()
        .route("/audit_logger", get(audit_logger))
        .route("/", get(|| async { "Hello, World!" }))
        .with_state(ctx);
    tracing::info!("Server started on {}", &ip_prot);
    let listener = tokio::net::TcpListener::bind(ip_prot).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn audit_logger(state: State<Arc<Context>>) -> Json<Vec<TransactionEvent>> {
    let v = { state.audit_logger.log.lock().await.clone() };
    Json::from(v)
}
