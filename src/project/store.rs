use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

use tracing::{debug, instrument, trace};

use crate::project::{Project, ProjectId};

pub trait ProjectStore {
    fn get_by_id(&self, id: ProjectId) -> Option<Project>;

    fn save(&self, project: &Project);
}

#[derive(Debug, Clone)]
pub struct ProjectStoreInMemory {
    db: Rc<RefCell<HashMap<ProjectId, Project>>>,
}

impl ProjectStoreInMemory {
    #[instrument]
    pub fn new() -> Self {
        let project_store = Self {
            db: Rc::new(RefCell::new(HashMap::new())),
        };
        trace!("in-memory project store initialized");
        project_store
    }
}

impl ProjectStore for ProjectStoreInMemory {
    #[instrument(skip(self))]
    fn get_by_id(&self, id: ProjectId) -> Option<Project> {
        let maybe_project = self.db.borrow().get(&id).cloned();
        trace!("project read");
        maybe_project
    }

    #[instrument(skip(self))]
    fn save(&self, project: &Project) {
        self.db.borrow_mut().insert(project.id, project.clone());
        debug!("project saved");
    }
}
