use crate::audit_logger::AuditLogger;
use std::sync::Arc;
use crate::coordinator::default_coordinator::DefaultCoordinator;

pub(crate) struct Context {
    pub(crate) coordinator: Arc<DefaultCoordinator>,
    pub(crate) audit_logger: Arc<AuditLogger>,
}
impl Context {
    pub fn new_arc(
        coordinator: Arc<DefaultCoordinator>,
        audit_logger: Arc<AuditLogger>,
    ) -> Arc<Self> {
        Arc::new(Context {
            coordinator,
            audit_logger,
        })
    }
}
