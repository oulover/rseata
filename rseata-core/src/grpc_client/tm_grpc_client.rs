use crate::grpc_client::lazy::{LazyState, LazyStateInit};
use crate::grpc_client::{GrpcClient, GrpcContext, RseataInterceptor};
use async_trait::async_trait;
use rseata_proto::rseata_proto::proto::transaction_manager_service_client::TransactionManagerServiceClient;
use std::str::FromStr;
use tonic::codegen::InterceptedService;
use tonic::transport::{Channel, Endpoint};

pub type LazyTMGrpcClient = LazyState<GrpcClient<TMGrpcClient>, RseataInterceptor>;

#[derive(Clone)]
pub struct TMGrpcClient {
    pub tc: TransactionManagerServiceClient<InterceptedService<Channel, RseataInterceptor>>,
}
#[async_trait]
impl LazyStateInit for GrpcClient<TMGrpcClient> {
    type Error = anyhow::Error;
    type Context = GrpcContext;
    type InterceptorType = RseataInterceptor;

    async fn init(ctx: &Self::Context) -> Result<Self, Self::Error> {
        let channel = Endpoint::from_str(&ctx.endpoint)
            .map_err(|e| anyhow::Error::msg(e.to_string()))?
            .connect()
            .await?;
        let client = TransactionManagerServiceClient::with_interceptor(channel, RseataInterceptor);
        Ok(GrpcClient(TMGrpcClient { tc: client }))
    }
}
