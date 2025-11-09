use crate::grpc_service::TCGrpcService;
use crate::resource::TCResource;
use crate::types::ConnectionId;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use rseata_core::branch::branch_manager_outbound::BranchManagerOutbound;
use rseata_core::resource::DefaultResource;
use rseata_core::resource::resource_registry::ResourceRegistry;
use rseata_proto::rseata_proto::proto::resource_manager_service_server::ResourceManagerService;
use rseata_proto::rseata_proto::proto::{
    BaseResponse, BranchRegisterRequest, BranchRegisterResponse, BranchReportRequest,
    BranchReportResponse, LockQueryRequest, LockQueryResponse, ResourceInstruction, ResourceProto,
    UnregisterResourceResponse,
};
use std::pin::Pin;
use tokio::sync::mpsc;
use tonic::codegen::tokio_stream::Stream;
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};
use uuid::Uuid;

pub type RegisterResourceStreamType =
    Pin<Box<dyn Stream<Item = Result<ResourceInstruction, Status>> + Send + 'static>>;
#[async_trait]
impl ResourceManagerService for TCGrpcService {
    type RegisterResourceStream = RegisterResourceStreamType;

    async fn register_resource(
        &self,
        request: Request<Streaming<ResourceProto>>,
    ) -> std::result::Result<Response<Self::RegisterResourceStream>, Status> {
        let mut incoming_stream = request.into_inner();
        let (response_tx, response_rx) = mpsc::channel::<Result<ResourceInstruction, Status>>(128);

        let (request_tx, request_rx) = mpsc::channel::<Result<ResourceProto>>(128);

        let cm = self.coordinator.clone();

        tokio::spawn(async move {
            loop {
                let response_tx_cloned = response_tx.clone();
                match incoming_stream.message().await {
                    Ok(Some(resource_msg)) => {
                        tracing::debug!("--------A---resource_msg: {:?}", resource_msg); 
                        let resource_info = DefaultResource {
                            group_id: resource_msg.resource_group_id.clone(),
                            resource_id: resource_msg.resource_id.clone().into(),
                            branch_type: resource_msg.branch_type.into(),
                            client_id: resource_msg.client_id.into(),
                        };
                        let resource_holder = TCResource {
                            connection_id: ConnectionId::from(Uuid::new_v4().as_u128() as u64),
                            resource: resource_info,
                            response_tx: response_tx_cloned,
                        };

                        // 判断注册成功否？  失败则发送消息？
                        cm.register_resource(&resource_holder).await;
                    }
                    Ok(None) => {
                        tracing::debug!("Resource stream ended normally for connection ",);
                        request_tx
                            .send(Err(anyhow!("RegisterResourceStream ended!")))
                            .await
                            .ok();
                        break;
                    }
                    Err(e) => {
                        eprintln!(
                            "Error receiving branch register request for connection {}",
                            e
                        );
                        request_tx
                            .send(Err(anyhow!("RegisterResourceStream ended!")))
                            .await
                            .ok();
                        break;
                    }
                }
            }

            // 连接关闭时清理资源
            {
                request_tx.closed().await;
                // let mut managers = registered_managers.write().await;
                // let closed = managers
                //     .values()
                //     .flatten()
                //     .find(|e| e.connection_id == connection_id);
                // if let Some(closed) = closed {
                //     let rm_id = ResourceId(closed.resource_id.clone());
                //     let rms = managers.get_mut(&rm_id);
                //     if let Some(rm_vec) = rms {
                //         let find = rm_vec.iter().find(|e| e.connection_id == connection_id);
                //         if let Some(r) = find {
                //             rm_vec.retain(|r| r.connection_id != connection_id);
                //         }
                //     }
                // }
            }

            tracing::debug!("Branch register stream processing finished for connection ");
        });

        let output_stream = ReceiverStream::new(response_rx);
        tracing::debug!("register_resource registered");
        Ok(Response::new(
            Box::pin(output_stream) as Self::RegisterResourceStream
        ))
    }

    async fn unregister_resource(
        &self,
        request: Request<ResourceProto>,
    ) -> std::result::Result<Response<UnregisterResourceResponse>, Status> {
        todo!()
    }

    async fn branch_register(
        &self,
        request: Request<BranchRegisterRequest>,
    ) -> std::result::Result<Response<BranchRegisterResponse>, Status> {
        tracing::debug!("branch_register----{:?}", request);
        let request = request.into_inner();
        let branch_id = self
            .coordinator
            .branch_register(
                request.branch_type.into(),
                request.resource_id.into(),
                request.client_id.into(),
                request.xid.into(),
                request.application_data,
                request.lock_keys,
            )
            .await
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        Ok(Response::new(BranchRegisterResponse {
            branch_id: branch_id.into(),
            base: BaseResponse::success().some(),
        }))
    }

    async fn branch_report(
        &self,
        request: Request<BranchReportRequest>,
    ) -> std::result::Result<Response<BranchReportResponse>, Status> {
        let request = request.into_inner();
        self.coordinator
            .branch_report(
                request.branch_type.into(),
                request.xid.into(),
                request.branch_id.into(),
                request.status.into(),
                request.application_data,
            )
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(BranchReportResponse {}))
    }

    async fn lock_query(
        &self,
        request: Request<LockQueryRequest>,
    ) -> std::result::Result<Response<LockQueryResponse>, Status> {
        let request = request.into_inner();
        tracing::debug!("lock_query----{:?}", request);

        let lucked = self
            .coordinator
            .lock_query(
                request.branch_type.into(),
                request.resource_id.into(),
                request.xid.into(),
                request.lock_keys,
            )
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(LockQueryResponse {
            locked: lucked,
            base: BaseResponse::success().some(),
        }))
    }
}
