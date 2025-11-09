use crate::grpc_service::TCGrpcService;
use async_trait::async_trait;
use rseata_core::transaction::transaction_manager::TransactionManager;
use rseata_core::types::GlobalStatus;
use rseata_proto::rseata_proto::proto::transaction_manager_service_server::TransactionManagerService;
use rseata_proto::rseata_proto::proto::{
    BaseResponse, GlobalBeginRequest, GlobalBeginResponse, GlobalCommitRequest,
    GlobalCommitResponse, GlobalReportRequest, GlobalReportResponse, GlobalRollbackRequest,
    GlobalRollbackResponse, GlobalStatusRequest, GlobalStatusResponse,
};

#[async_trait]
impl TransactionManagerService for TCGrpcService {
    async fn global_begin(
        &self,
        request: tonic::Request<GlobalBeginRequest>,
    ) -> Result<tonic::Response<GlobalBeginResponse>, tonic::Status> {
        tracing::debug!("global_begin ----: {:?}", request);
        let request = request.into_inner();

        let xid = self
            .coordinator
            .begin(
                request.application_id,
                request.transaction_name,
                request.transaction_service_group,
                request.timeout_millis,
            )
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        Ok(tonic::Response::new(GlobalBeginResponse {
            xid: xid.0,
            base: BaseResponse::success().some(),
        }))
    }

    async fn global_commit(
        &self,
        request: tonic::Request<GlobalCommitRequest>,
    ) -> Result<tonic::Response<GlobalCommitResponse>, tonic::Status> {
        tracing::debug!("global_commit ----: {:?}", request);
        let request = request.into_inner();
        let status = self
            .coordinator
            .commit(request.xid.into())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        Ok(tonic::Response::new(GlobalCommitResponse {
            global_status: status.code(),
            base: BaseResponse::success().some(),
        }))
    }

    async fn global_rollback(
        &self,
        request: tonic::Request<GlobalRollbackRequest>,
    ) -> Result<tonic::Response<GlobalRollbackResponse>, tonic::Status> {
        tracing::debug!("global_rollback ----: {:?}", request);
        let request = request.into_inner();
        let status = self
            .coordinator
            .rollback(request.xid.into())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        Ok(tonic::Response::new(GlobalRollbackResponse {
            global_status: status.code(),
            base: BaseResponse::success().some(),
        }))
    }

    async fn get_global_status(
        &self,
        request: tonic::Request<GlobalStatusRequest>,
    ) -> Result<tonic::Response<GlobalStatusResponse>, tonic::Status> {
        tracing::debug!("get_global_status ----: {:?}", request);
        let request = request.into_inner();
        let status = self
            .coordinator
            .get_status(request.xid.into())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        Ok(tonic::Response::new(GlobalStatusResponse {
            global_status: status.code(),
            base: BaseResponse::success().some(),
        }))
    }

    async fn global_report(
        &self,
        request: tonic::Request<GlobalReportRequest>,
    ) -> Result<tonic::Response<GlobalReportResponse>, tonic::Status> {
        tracing::debug!("global_report ----: {:?}", request);
        let request = request.into_inner();
        let status = self
            .coordinator
            .global_report(
                &request.xid.into(),
                GlobalStatus::from_code(request.global_status).unwrap(),
            )
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        Ok(tonic::Response::new(GlobalReportResponse {
            global_status: status.code(),
            base: BaseResponse::success().some(),
        }))
    }
}
