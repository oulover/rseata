use async_trait::async_trait;
use crate::grpc_client::lazy::{LazyState, LazyStateInit};
use crate::grpc_client::{GrpcClient, RseataInterceptor};
use rseata_proto::rseata_proto::proto::resource_manager_service_client::ResourceManagerServiceClient;
use tonic::codegen::InterceptedService;
use tonic::transport::{Channel, Endpoint};

pub type LazyRMGrpcClient = LazyState<GrpcClient<RMGrpcClient>, RseataInterceptor>;

#[derive(Clone)]
pub struct RMGrpcClient {
    pub rm: ResourceManagerServiceClient<InterceptedService<Channel, RseataInterceptor>>,
}
#[async_trait]
impl LazyStateInit for GrpcClient<RMGrpcClient> {
    type Error = anyhow::Error;
    type Context = ();
    type InterceptorType = RseataInterceptor;

    async fn init(_: &Self::Context) -> Result<Self, Self::Error> {
        let channel = Endpoint::from_static("http://127.0.0.1:9811")
            .connect()
            .await?;
        let client = ResourceManagerServiceClient::with_interceptor(channel, RseataInterceptor);
        Ok(GrpcClient(RMGrpcClient { rm: client }))
    }
}
