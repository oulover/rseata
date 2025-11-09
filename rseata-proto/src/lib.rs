use crate::rseata_proto::proto::BaseResponse;

pub mod rseata_proto {
    pub mod proto {
        tonic::include_proto!("cn.olbe.grpc.rust.seata.core");
    }
}
const BASE_RESPONSE_SUCCESS: u32 = 0;
impl BaseResponse {
    pub fn is_success(&self) -> bool {
        self.result_code == BASE_RESPONSE_SUCCESS
    }
    pub fn is_failed(&self) -> bool {
        !self.is_success()
    }
    pub fn some(self: Self) -> Option<Self> {
        Some(self)
    }
}
impl BaseResponse {
    fn new(result_code: u32, message: Option<&str>) -> Self {
        Self {
            result_code,
            message: message.map(|s| s.to_string()),
        }
    }
    pub fn success() -> Self {
        Self::new(BASE_RESPONSE_SUCCESS, None)
    }
    pub fn failed() -> Self {
        Self::new(500, None)
    }
    pub fn failed_with_msg(message: &str) -> Self {
        Self::failed_with_code_msg(500, message)
    }
    pub fn failed_with_code_msg(result_code: u32, message: &str) -> Self {
        Self::new(result_code, Some(message))
    }
}
