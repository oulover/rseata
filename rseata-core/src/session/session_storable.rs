use crate::error::TransactionError;

pub trait SessionStorable: Send + Sync {
    /// Encode the session to bytes for storage
    fn encode(&self) -> Result<Vec<u8>, TransactionError>;

    /// Decode the session from bytes
    fn decode(data: &[u8]) -> Result<Self, TransactionError>
    where
        Self: Sized;

    /// Check if the serialized size exceeds the maximum limit
    fn check_size(&self) -> Result<(), TransactionError> {
        // Default implementation, can be overridden
        let encoded = self.encode()?;
        if encoded.len() > self.max_size() {
            return Err(TransactionError::new(format!(
                "Session size exceeded: {} > {}",
                encoded.len(),
                self.max_size()
            )));
        }
        Ok(())
    }

    /// Get the maximum allowed size for this session type
    fn max_size(&self) -> usize;
}
