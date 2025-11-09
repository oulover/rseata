use crate::error::TransactionError;
use async_trait::async_trait;

#[async_trait]
pub trait Lockable: Send + Sync {
    async fn lock(&self) -> Result<bool, TransactionError>;
    async fn unlock(&self) -> Result<bool, TransactionError>;
}
