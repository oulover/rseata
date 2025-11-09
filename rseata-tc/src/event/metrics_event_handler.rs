// use std::sync::Arc;
// use async_trait::async_trait;
//
// pub struct MetricsEventHandler {
//     metrics: Arc<MetricsCollector>,
// }
//
// impl MetricsEventHandler {
//     pub fn new(metrics: Arc<MetricsCollector>) -> Self {
//         Self { metrics }
//     }
// }
//
// #[async_trait]
// impl EventHandler for MetricsEventHandler {
//     fn name(&self) -> &str {
//         "metrics_event_handler"
//     }
//
//     async fn handle_event(&self, event: &TransactionEvent) -> Result<(), SeataError> {
//         match &event.event_type {
//             TransactionEventType::GlobalBegin { timeout_millis } => {
//                 self.metrics.increment_global_transactions();
//                 self.metrics.record_transaction_timeout(*timeout_millis);
//             }
//             TransactionEventType::GlobalCommit { status, commit_duration_millis } => {
//                 self.metrics.record_commit_duration(*commit_duration_millis);
//                 if *status == GlobalStatus::Committed {
//                     self.metrics.increment_successful_commits();
//                 } else {
//                     self.metrics.increment_failed_commits();
//                 }
//             }
//             TransactionEventType::BranchRegister { branch_type, .. } => {
//                 self.metrics.increment_branch_transactions(*branch_type);
//             }
//             _ => {}
//         }
//         Ok(())
//     }
//
//     fn interested_event_types(&self) -> Vec<TransactionEventType> {
//         vec![
//             TransactionEventType::GlobalBegin { timeout_millis: 0 },
//             TransactionEventType::GlobalCommit { status: GlobalStatus::Begin, commit_duration_millis: 0 },
//             TransactionEventType::GlobalRollback { status: GlobalStatus::Begin, rollback_duration_millis: 0 },
//             TransactionEventType::BranchRegister { branch_id: 0, branch_type: BranchType::AT, resource_id: "".to_string(), lock_keys: "".to_string() },
//         ]
//     }
// }