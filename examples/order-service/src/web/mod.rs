use crate::context::AppContext;
use crate::service;
use crate::web::handler::handle_result;
use axum::Router;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::routing::get;
use std::sync::Arc;

pub(crate) async fn start(app_ctx: Arc<AppContext>) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/add_order_then_add_user", get(get_user_info))
        .with_state(app_ctx);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:4002").await?;
    tracing::info!("Server started on http://0.0.0.0:4002");
    axum::serve(listener, app).await?;
    Ok(())
}

pub async fn get_user_info(
    state: State<Arc<AppContext>>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let r = service::order_service::add_order_then_add_user(state.0).await;
    handle_result(r)
}

pub mod handler {
    use axum::Json;
    use axum::http::StatusCode;
    use axum::response::{IntoResponse, Response};
    use serde::Serialize;
    use thiserror::Error;

    #[derive(Serialize)]
    pub struct R<T> {
        pub code: u16,
        pub msg: String,
        pub data: T,
    }

    #[derive(Error, Debug)]
    pub enum Error {
        #[error("inner: {inner}")]
        Inner { inner: String },
    }

    impl From<anyhow::Error> for Error {
        fn from(value: anyhow::Error) -> Self {
            Self::Inner {
                inner: value.to_string(),
            }
        }
    }

    impl IntoResponse for Error {
        fn into_response(self) -> Response {
            let code = StatusCode::INTERNAL_SERVER_ERROR;
            let message = self.to_string();
            let body = Json(R {
                code: code.as_u16(),
                msg: message,
                data: (),
            });
            (code, body).into_response()
        }
    }
    pub fn handle_result<T: Serialize>(
        result: anyhow::Result<T>,
    ) -> Result<impl IntoResponse, Error> {
        match result {
            Ok(data) => {
                let response = R {
                    code: StatusCode::OK.as_u16(),
                    msg: "success".to_string(),
                    data,
                };
                Ok(Json(response))
            }
            Err(e) => Err(Error::from(e)),
        }
    }
}
