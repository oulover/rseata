use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransactionError {
    #[error("Unknown Error")]
    Unknown,
    #[error("context: missing '{info}'")]
    ErrorInfo { info: String },
}

impl TransactionError {
    pub fn new(e: String) -> Self {
        TransactionError::ErrorInfo { info: e }
    }
}
