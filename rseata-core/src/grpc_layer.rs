use crate::session::ClientSession;
use crate::types::Xid;
use crate::{RSEATA_CLIENT_SESSION, RSEATA_XID_KEY};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tonic::codegen::{http, Service};
use tower::Layer;

#[derive(Debug, Clone)]
pub struct SeataMiddlewareLayer;

impl<S> Layer<S> for SeataMiddlewareLayer {
    type Service = ServerMiddleware<S>;

    fn layer(&self, service: S) -> Self::Service {
        ServerMiddleware { inner: service }
    }
}

#[derive(Clone)]
pub struct ServerMiddleware<S> {
    inner: S,
}

impl<S, ReqBody> Service<http::Request<ReqBody>> for ServerMiddleware<S>
where
    S: Service<http::Request<ReqBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: http::Request<ReqBody>) -> Self::Future {
        let mut inner = self.inner.clone();
        // 从请求的metadata中提取xid
        let xid = req
            .headers()
            .get(RSEATA_XID_KEY)
            .and_then(|value| value.to_str().ok())
            .map(|s| s.to_string());

        if let Some(xid) = xid {
            let xid: Xid = xid.into();
            let session = Arc::new(ClientSession::new(String::new()));
            session.begin_global_transaction(xid).unwrap();
            Box::pin(async move {
                RSEATA_CLIENT_SESSION
                    .scope(session, async move { inner.call(req).await })
                    .await
            })
        } else {
            Box::pin(async move { inner.call(req).await })
        }
    }
}
