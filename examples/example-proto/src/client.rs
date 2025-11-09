use crate::lazy::{GrpcClient, LazyState, LazyStateInit};
use crate::rseata_proto::proto::user_service_client::UserServiceClient;
use rseata_core::grpc_client::RseataInterceptor;
use std::str::FromStr;
use tonic::codegen::InterceptedService;
use tonic::transport::{Channel, Endpoint};

pub struct GrpcCtx {
    pub endpoint: String,
}

pub type LazyTestUserGrpcClient = LazyState<GrpcClient<UserGrpcClient>, RseataInterceptor>;
#[derive(Clone, Debug)]
pub struct UserGrpcClient {
    pub user: UserServiceClient<InterceptedService<Channel, RseataInterceptor>>,
}

impl LazyStateInit for GrpcClient<UserGrpcClient> {
    type Error = anyhow::Error;
    type Context = GrpcCtx;
    type InterceptorType = RseataInterceptor;

    async fn init(ctx: &Self::Context) -> Result<Self, Self::Error> {
        let channel = Endpoint::from_str(&ctx.endpoint)?.connect().await?;
        let client = UserServiceClient::with_interceptor(channel, RseataInterceptor);
        Ok(GrpcClient(UserGrpcClient { user: client }))
    }
}
