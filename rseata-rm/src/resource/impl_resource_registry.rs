use async_trait::async_trait;
use tokio::sync::mpsc;
use tonic::codegen::tokio_stream::StreamExt;
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use rseata_core::branch::branch_manager_inbound::BranchManagerInbound;
use rseata_core::resource::Resource;
use rseata_core::resource::resource_registry::ResourceRegistry;
use rseata_proto::rseata_proto::proto::resource_instruction::Instruction;
use rseata_proto::rseata_proto::proto::ResourceProto;
use crate::resource::{DefaultResourceManager, ResourceInfo};

#[async_trait]
impl ResourceRegistry for DefaultResourceManager {
    type Resource = ResourceInfo;

    async fn register_resource(&self, resource: &Self::Resource) {
        let resource_clone = resource.clone();
        // 注册流到远程
        {
            let (request_tx, request_rx) = mpsc::channel(100);
            let (response_tx, response_rx) = mpsc::channel(100);

            let request_stream = ReceiverStream::new(request_rx);

            // 启动持久的流调用
            let response_stream = self
                .rm_client
                .get()
                .await
                .unwrap()
                .rm
                .register_resource(tonic::Request::new(request_stream))
                .await
                .unwrap()
                .into_inner();

            let self_cloned = self.clone();
            // 处理响应流的后台任务
            tokio::spawn(async move {
                let mut response_stream = response_stream;
                while let Some(response) = response_stream.next().await {
                    match response {
                        Ok(branch_response) => {
                            if let Some(instruction) = branch_response.instruction {
                                tracing::info!("------------Received instruction: {:?}", instruction);
                                match instruction {
                                    Instruction::Commit(commit) => {
                                        self_cloned
                                            .branch_commit(
                                                commit.branch_type.into(),
                                                commit.xid.into(),
                                                commit.branch_id.into(),
                                                commit.resource_id.into(),
                                                commit.application_data,
                                            )
                                            .await
                                            .ok();
                                    }
                                    Instruction::Rollback(rollback) => {
                                        self_cloned
                                            .branch_rollback(
                                                rollback.branch_type.into(),
                                                rollback.xid.into(),
                                                rollback.branch_id.into(),
                                                rollback.resource_id.into(),
                                                rollback.application_data,
                                            )
                                            .await
                                            .ok();
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Error in branch register stream: {}", e);
                            break;
                        }
                    }
                }
            });

            // Resource Registry
            {
                let _ = request_tx
                    .send(ResourceProto {
                        resource_group_id: resource_clone.resource_group_id,
                        resource_id: resource_clone.resource_id.0,
                        client_id: resource_clone.client_id.into(),
                        branch_type: resource_clone.branch_type.into(),
                    })
                    .await;
            }

            // 添加本地
            let mut channel = self.channel.write().await;
            *channel = Some((request_tx, response_rx));
        }

        // 添加本地
        let mut resources = self.resources.write().await;
        resources.insert(resource.get_resource_id().await, Box::new(resource.clone()));
    }

    async fn unregister_resource(&mut self, resource: &Self::Resource) {}
}