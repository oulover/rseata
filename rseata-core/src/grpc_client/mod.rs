use crate::{RSEATA_CLIENT_SESSION, RSEATA_XID_KEY};
use std::str::FromStr;
use tonic::metadata::{MetadataKey, MetadataValue};
use tonic::service::Interceptor;
use tonic::{Code, Request, Status};

pub mod grpc_client_impl;
pub mod lazy;
pub mod source;

/// Rust does not allow impl LazyStateInit for structs from external crates
#[derive(Clone)]
pub struct GrpcClient<T>(pub T);
impl<T> std::ops::Deref for GrpcClient<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for GrpcClient<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone)]
pub struct RseataInterceptor;
impl Interceptor for RseataInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        if let Some(started) = RSEATA_CLIENT_SESSION
            .try_with(|v| v.is_global_tx_started())
            .ok()
        {
            if started {
                let xid1 = RSEATA_CLIENT_SESSION.try_with(|s| s.get_xid()).ok();
                if let Some(xid2) = xid1 {
                    if let Some(xid3) = xid2 {
                        if let Ok(metadata_key) = MetadataKey::from_bytes(RSEATA_XID_KEY.as_bytes())
                        {
                            if let Ok(metadata_value) = MetadataValue::from_str(&xid3.to_string()) {
                                request.metadata_mut().insert(metadata_key, metadata_value);
                            } else {
                                return Err(Status::new(
                                    Code::InvalidArgument,
                                    "RSEATA_XID is invalid",
                                ));
                            }
                        } else {
                            return Err(Status::new(
                                Code::InvalidArgument,
                                "RSEATA_XID_KEY is invalid",
                            ));
                        }
                    }
                } else {
                    return Err(Status::new(
                        Code::InvalidArgument,
                        "RSEATA_XID_KEY is invalid",
                    ));
                }
            }
        }
        Ok(request)
    }
}
