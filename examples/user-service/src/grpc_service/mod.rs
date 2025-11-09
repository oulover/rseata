use crate::context::AppContext;
use crate::user::user::Model;
use crate::{service, user};
use example_proto::rseata_proto::proto::user_service_server::{UserService, UserServiceServer};
use example_proto::rseata_proto::proto::{
    AddUserRequest, AddUserResponse, GetUserByIdRequest, GetUserByIdResponse, User,
};
use rseata::RSEATA_CLIENT_SESSION;
use rseata::core::GrpcServerInterceptor;
use rseata::core::grpc_layer::SeataMiddlewareLayer;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::sqlx::types::uuid;
use sea_orm::{DbErr, EntityTrait, TransactionTrait};
use std::pin::Pin;
use std::sync::Arc;
use tonic::transport::Server;
use uuid::uuid;

pub(crate) async fn start(ctx: Arc<AppContext>) -> anyhow::Result<()> {
    let addr = std::env::var("GRPC_BIND")
        .unwrap_or_else(|_| "0.0.0.0:9001".into())
        .parse()?;
    tracing::info!("Server started on 0.0.0.0:9001");
    Server::builder()
        .layer(SeataMiddlewareLayer)
        .add_service(UserServiceServer::new(UserGrpcService { app_ctx: ctx }))
        .serve(addr)
        .await?;
    Ok(())
}
pub struct UserGrpcService {
    pub app_ctx: Arc<AppContext>,
}
#[async_trait]
impl UserService for UserGrpcService {
    async fn get_user_by_id(
        &self,
        request: tonic::Request<GetUserByIdRequest>,
    ) -> Result<tonic::Response<GetUserByIdResponse>, tonic::Status> {
        tracing::info!("Got a request: {:?}", request.into_inner());
        let r = RSEATA_CLIENT_SESSION.try_get().ok();
        tracing::info!("UserService Got a session {:?}", r);

        let find_user = user::user::Entity::find_by_id(1)
            .one(&self.app_ctx.db_conn)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        Ok(tonic::Response::new(GetUserByIdResponse {
            user: find_user.map(|u| User {
                id: u.id as u64,
                name: u.name,
                age: u.age,
                sex: u.sex,
            }),
        }))
    }

    async fn add_user(
        &self,
        request: tonic::Request<AddUserRequest>,
    ) -> Result<tonic::Response<AddUserResponse>, tonic::Status> {
        let request = request.into_inner();
        tracing::info!("Got a request: {:?}", request);
        let r = RSEATA_CLIENT_SESSION.try_get().ok();
        tracing::info!("UserService Got a session {:?}", r);

        let user_add = service::user_service::add_user(
            self.app_ctx.clone(),
            Model {
                id: uuid::Uuid::new_v4().as_u128() as i64,
                name: request.name,
                age: request.age,
                sex: request.sex,
            },
        )
        .await
        .map_err(|e| tonic::Status::internal(e.to_string()))?;

        Ok(tonic::Response::new(AddUserResponse {
            user: Some(User {
                id: user_add.id as u64,
                name: user_add.name,
                age: user_add.age,
                sex: user_add.sex,
            }),
        }))
    }
}
