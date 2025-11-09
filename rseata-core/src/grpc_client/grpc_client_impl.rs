use async_trait::async_trait;
use crate::grpc_client::lazy::{LazyState, LazyStateInit};
use crate::grpc_client::{GrpcClient, RseataInterceptor};
use rseata_proto::rseata_proto::proto::transaction_manager_service_client::TransactionManagerServiceClient;
use tonic::codegen::InterceptedService;
use tonic::transport::{Channel, Endpoint};

pub type LazyTmGrpcClient = LazyState<GrpcClient<TmGrpcClient>, RseataInterceptor>;

#[derive(Clone)]
pub struct TmGrpcClient {
    pub tc: TransactionManagerServiceClient<InterceptedService<Channel, RseataInterceptor>>,
}
#[async_trait]
impl LazyStateInit for GrpcClient<TmGrpcClient> {
    type Error = anyhow::Error;
    type Context = ();
    type InterceptorType = RseataInterceptor;

    async fn init(_: &Self::Context) -> Result<Self, Self::Error> {
        let channel = Endpoint::from_static("http://127.0.0.1:9811")
            .connect()
            .await?;
        let client = TransactionManagerServiceClient::with_interceptor(channel, RseataInterceptor);
        Ok(GrpcClient(TmGrpcClient { tc: client }))
    }
}
