// use crate::error::TransactionError;
// use crate::resource::BranchStatus;
// use crate::session::branch_session::BranchSession;
// use crate::types::GlobalStatus;
// use async_trait::async_trait;
// 
// #[async_trait]
// pub trait SessionLifecycle: Send + Sync {
//     type BranchSession: BranchSession + Send + Sync;
//     async fn begin(&mut self) -> Result<(), TransactionError>;
//     async fn change_global_status(&self, status: GlobalStatus) -> Result<(), TransactionError>;
//     async fn change_branch_status(
//         &self,
//         branch_session: &Self::BranchSession,
//         status: BranchStatus,
//     ) -> Result<(), TransactionError>;
//     async fn add_branch(
//         &self,
//         branch_session: Self::BranchSession,
//     ) -> Result<(), TransactionError>;
//     async fn unlock_branch(
//         &self,
//         branch_session: &Self::BranchSession,
//     ) -> Result<(), TransactionError>;
//     async fn remove_branch(
//         &self,
//         branch_session: &Self::BranchSession,
//     ) -> Result<(), TransactionError>;
//     async fn remove_and_unlock_branch(
//         &self,
//         branch_session: &Self::BranchSession,
//     ) -> Result<(), TransactionError>;
//     fn is_active(&self) -> bool;
//     async fn close(&mut self) -> Result<(), TransactionError>;
//     async fn end(&self) -> Result<(), TransactionError>;
// }
