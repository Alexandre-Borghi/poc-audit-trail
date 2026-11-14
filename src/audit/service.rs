use std::rc::Rc;

use tracing::{debug, instrument, trace};

use super::{AuditStore, Record};

#[derive(Clone)]
pub struct AuditService {
    audit_store: Rc<dyn AuditStore>,
}

impl AuditService {
    #[instrument(skip_all)]
    pub fn new(audit_store: impl AuditStore + 'static) -> Self {
        let audit_service = Self {
            audit_store: Rc::new(audit_store),
        };
        trace!("audit service initialized");
        audit_service
    }

    #[instrument(skip(self))]
    pub fn record(&self, record: &Record) {
        self.audit_store.save(record);
        debug!("audit record recorded");
    }

    #[instrument(skip(self))]
    pub fn list_all(&self) -> Vec<Record> {
        let all_records = self.audit_store.get_all();
        debug!("audit records listed");
        all_records
    }
}
