use thiserror::Error;

#[derive(Error, Debug)]
pub enum EventError {
    #[error("Unknown Error")]
    Unknown,
    #[error("context: missing '{info}'")]
    ErrorInfo { info: String },
}

impl EventError {
    pub(crate) fn new(e: String) -> Self {
        EventError::ErrorInfo { info: e }
    }
}
