use crate::audit_logger::AuditLogger;
use crate::context::Context;
use crate::event::audit_log_event_handler::AuditLogEventHandler;
use rseata_core::event::defaults::default_event_handler_chain::DefaultEventHandlerChain;
use rseata_core::event::defaults::event_publisher::DefaultEventPublisher;
use rseata_core::event::event_handler_chain::EventHandlerChain;
use rseata_core::session::defaults::default_session_manager::DefaultSessionManager;
use rseata_core::store::memery_transaction_store_manager::MemeryTransactionStoreManager;
use std::sync::Arc;
use crate::coordinator::default_coordinator::DefaultCoordinator;

mod audit_logger;
mod config;
mod context;
pub(crate) mod coordinator;
pub(crate) mod error;
mod event;
pub(crate) mod grpc_service;
pub(crate) mod init;
pub(crate) mod resource;
pub(crate) mod types;
pub(crate) mod web;

pub async fn start_server() -> anyhow::Result<()> {
    init::init().await?;
    let session_manager = Arc::new(DefaultSessionManager::new(
        String::from("DefaultSessionManager"),
        Box::new(MemeryTransactionStoreManager::default()),
    ));

    let (event_publisher, audit_logger) = start_event_system().await;

    let coordinator_manager = Arc::new(DefaultCoordinator::new(event_publisher));
    let ctx = Context::new_arc(coordinator_manager, audit_logger);
    tokio::spawn(grpc_service::start(ctx.clone()));
    web::start(ctx).await?;
    Ok(())
}

pub async fn start_event_system() -> (Arc<DefaultEventPublisher>, Arc<AuditLogger>) {
    let mut handler_chain = DefaultEventHandlerChain::new();

    let audit_logger = Arc::new(AuditLogger {
        log: Arc::new(Default::default()),
    });
    let audit_handler = Arc::new(AuditLogEventHandler::new(audit_logger.clone()));
    handler_chain.add_handler(audit_handler).await;

    (
        Arc::new(DefaultEventPublisher::new(Arc::new(handler_chain))),
        audit_logger,
    )
}
