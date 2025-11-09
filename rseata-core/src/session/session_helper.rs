// use uuid::Uuid;
// use async_trait::async_trait;
// use crate::error::TransactionError;
// use crate::resource::{BranchStatus, BranchType};
// use crate::session::branch_session::BranchSession;
// use crate::session::defaults::default_branch_session::DefaultBranchSession;
// use crate::session::defaults::default_global_session::DefaultGlobalSession;
// use crate::session::session_life_cycle::SessionLifecycle;
// use crate::types::{GlobalStatus, Xid};
// 
// pub struct SessionHelper;
// 
// impl SessionHelper {
//     
//     pub fn new_branch_by_global(
//         global_session: &DefaultGlobalSession,
//         branch_type: BranchType,
//         resource_id: &str,
//         lock_keys: &str,
//         client_id: &str,
//     ) -> DefaultBranchSession {
//         Self::new_branch_by_global_with_data(global_session, branch_type, resource_id, None, lock_keys, client_id)
//     }
// 
//     pub fn new_branch_by_global_with_data(
//         global_session: &DefaultGlobalSession,
//         branch_type: BranchType,
//         resource_id: &str,
//         application_data: Option<&str>,
//         lock_keys: &str,
//         client_id: &str,
//     ) -> DefaultBranchSession {
//         let mut branch_session = DefaultBranchSession::new(branch_type);
// 
//         branch_session.xid = global_session.xid.clone();
//         branch_session.transaction_id = global_session.transaction_id;
//         branch_session.branch_id = Uuid::new_v4().as_u128() as u64;
//         branch_session.resource_id = Some(resource_id.to_string());
//         branch_session.lock_key = Some(lock_keys.to_string());
//         branch_session.client_id = Some(client_id.to_string());
// 
//         if let Some(data) = application_data {
//             branch_session.application_data= Some(data.to_string());
//         }
// 
//         branch_session.status= BranchStatus::Registered;
//         branch_session
//     }
// 
//     pub fn new_branch(
//         branch_type: BranchType,
//         xid: &Xid,
//         branch_id: u64,
//         resource_id: &str,
//         application_data: Option<&str>,
//     ) -> DefaultBranchSession {
//         let mut branch_session = DefaultBranchSession::new(branch_type);
//         branch_session.xid=xid.clone();
//         branch_session.branch_id=(branch_id);
//         branch_session.resource_id=Some(resource_id.to_string());
// 
//         if let Some(data) = application_data {
//             branch_session.application_data=Some(data.to_string());
//         }
// 
//         branch_session
//     }
// 
//     pub async fn end_committed(
//         global_session: &DefaultGlobalSession,
//         retry_global: bool,
//     ) -> Result<(), TransactionError> {
//         // Implementation similar to Java version
//         if retry_global || !Self::delay_handle_session() {
//             let begin_time = std::time::SystemTime::now();
//             let retry_branch = global_session.status == GlobalStatus::CommitRetrying;
// 
//             if global_session.status != GlobalStatus::Committed {
//                 global_session.change_global_status(GlobalStatus::Committed).await?;
//             }
// 
//             global_session.end().await?;
// 
//             if !Self::delay_handle_session() {
//                 // MetricsPublisher::post_session_done_event(global_session, retry_global, false);
//             }
//             // Metrics publishing logic
//         } else {
//             // global_session.set_status(GlobalStatus::Committed);
//             if global_session.is_saga() {
//                 global_session.end().await?;
//             }
//             // Metrics publishing logic
//         }
//         Ok(())
//     }
// 
//     // pub async fn for_each_global_sessions<F>(
//     //     sessions: &[DefaultGlobalSession],
//     //     handler: F,
//     //     parallel: bool,
//     // ) where
//     //     F: GlobalSessionHandler + Send + Sync + Clone,
//     // {
//     //     if sessions.is_empty() {
//     //         return;
//     //     }
//     // 
//     //     if parallel {
//     //         let handles: Vec<_> = sessions
//     //             .iter()
//     //             .map(|session| {
//     //                 let handler = handler.clone();
//     //                 let session = session.clone();
//     //                 tokio::spawn(async move {
//     //                     // Set up context (MDC equivalent)
//     //                     handler.handle(&session).await;
//     //                 })
//     //             })
//     //             .collect();
//     // 
//     //         for handle in handles {
//     //             let _ = handle.await;
//     //         }
//     //     } else {
//     //         for session in sessions {
//     //             handler.handle(session).await;
//     //         }
//     //     }
//     // }
// 
//     // pub async fn for_each_branch_sessions<F>(
//     //     sessions: &[DefaultBranchSession],
//     //     handler: F,
//     // ) -> Result<Option<bool>, TransactionError>
//     // where
//     //     F: BranchSessionHandler + Send + Sync,
//     // {
//     //     for branch_session in sessions {
//     //         let result = handler.handle(branch_session).await?;
//     //         if result.is_some() {
//     //             return Ok(result);
//     //         }
//     //     }
//     //     Ok(None)
//     // }
//     // 
//     fn delay_handle_session() -> bool {
//         // Check session mode configuration
//         // !matches!(StoreConfig::session_mode(), SessionMode::File | SessionMode::Raft)
//         false
//     }
// }
// 
// // #[async_trait]
// // pub trait GlobalSessionHandler: Send + Sync {
// //     async fn handle(&self, global_session: &DefaultGlobalSession);
// // }
// // 
// // #[async_trait]
// // pub trait BranchSessionHandler: Send + Sync {
// //     async fn handle(&self, branch_session: &DefaultBranchSession) -> Result<Option<bool>, TransactionError>;
// // }