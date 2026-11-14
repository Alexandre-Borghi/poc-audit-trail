use std::{cell::RefCell, collections::HashMap, rc::Rc};

use tracing::{instrument, trace};

use super::{Record, RecordId};

pub trait AuditStore {
    fn save(&self, record: &Record);

    /// Returns all audit records, ordered by creation time.
    fn get_all(&self) -> Vec<Record>;
}

#[derive(Debug, Clone)]
pub struct AuditStoreInMemory {
    db: Rc<RefCell<HashMap<RecordId, Record>>>,
}

impl AuditStoreInMemory {
    #[instrument(skip_all)]
    pub fn new() -> Self {
        Self {
            db: Rc::new(RefCell::new(HashMap::new())),
        }
    }
}

impl AuditStore for AuditStoreInMemory {
    #[instrument(skip(self))]
    fn save(&self, record: &Record) {
        self.db.borrow_mut().insert(record.id, record.clone());
        trace!("record saved")
    }

    #[instrument(skip(self))]
    fn get_all(&self) -> Vec<Record> {
        let mut records = self.db.borrow().values().cloned().collect::<Vec<_>>();
        trace!(count = records.len(), "all records fetched");
        records.sort_by_key(|record| record.creation_time);
        trace!("records sorted");
        records
    }
}
