use crate::RSEATA_XID_KEY;
use tonic::service::Interceptor;
use tonic::{Request, Status};

#[derive(Debug, Clone)]
pub struct GrpcServerInterceptor;
impl Interceptor for GrpcServerInterceptor {
    fn call(&mut self, request: Request<()>) -> Result<Request<()>, Status> {
        let option = request.metadata().get(RSEATA_XID_KEY);
        if let Some(xid) = option {
            let xit = xid.to_str();
            match xit {
                Ok(x) => {
                    println!("GrpcServerInterceptor --- {}", x);
                }
                Err(e) => return Err(Status::invalid_argument(e.to_string())),
            }
        }
        Ok(request)
    }
}
