use crate::context::AppContext;
use axum::Router;
use axum::routing::get;
use std::sync::Arc;

pub(crate) async fn start(app_ctx: Arc<AppContext>) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/get/{id}", get(handler::user::get_user_info))
        .with_state(app_ctx);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:4001").await?;
    tracing::info!("Server started on http://0.0.0.0:4001");
    axum::serve(listener, app).await?;
    Ok(())
}

pub mod handler {
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct R<T> {
        pub code: u16,
        pub msg: String,
        pub data: T,
    }

    pub mod user {
        use crate::context::AppContext;
        use crate::service;
        use crate::web::handler::R;
        use axum::Json;
        use axum::extract::{Path, State};
        use axum::http::StatusCode;
        use axum::response::{IntoResponse, Response};
        use serde::Serialize;
        use std::sync::Arc;
        use thiserror::Error;

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

        pub async fn get_user_info(
            state: State<Arc<AppContext>>,
            Path(id): Path<i64>,
        ) -> Result<impl IntoResponse, impl IntoResponse> {
            let r = service::user_service::get_user(state.0, id).await;
            handle_result(r)
        }
    }
}
