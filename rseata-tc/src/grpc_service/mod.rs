mod rm_service;
mod tm_service;

use crate::Context;

use rseata_proto::rseata_proto::proto::resource_manager_service_server::ResourceManagerServiceServer;
use rseata_proto::rseata_proto::proto::transaction_manager_service_server::TransactionManagerServiceServer;

use std::sync::Arc;
use tonic::transport::Server;
use crate::coordinator::default_coordinator::DefaultCoordinator;

pub(crate) async fn start(ctx: Arc<Context>) -> anyhow::Result<()> {
    let addr = std::env::var("GRPC_BIND")
        .unwrap_or_else(|_| "0.0.0.0:9811".into())
        .parse()?;
    tracing::info!("Server started on 0.0.0.0:9811");

    let svc = TCGrpcService::new(ctx.coordinator.clone());

    Server::builder()
        .add_service(TransactionManagerServiceServer::new(svc.clone()))
        .add_service(ResourceManagerServiceServer::new(svc.clone()))
        //.add_service(ResourceManagerOutboundServiceServer::new(svc.clone()))
        .serve(addr)
        .await?;
    Ok(())
}

#[derive(Clone)]
pub(crate) struct TCGrpcService {
    coordinator: Arc<DefaultCoordinator>,
}

impl TCGrpcService {
    pub fn new(coordinator: Arc<DefaultCoordinator>) -> Self {
        Self {
            coordinator,
        }
    }
}
