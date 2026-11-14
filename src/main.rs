use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{Debug, Display},
    rc::Rc,
};

use tracing::{debug, info, instrument, trace};
use uuid::Uuid;

mod audit;

// --- Domain ---

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct ProjectId(Uuid);

#[derive(Debug, Clone)]
struct Project {
    id: ProjectId,
    r#ref: String,
    title: String,
}

impl Project {
    fn new(r#ref: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: ProjectId(Uuid::new_v4()),
            r#ref: r#ref.into(),
            title: title.into(),
        }
    }
}

impl Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.r#ref, self.title)
    }
}

// --- Application ---

struct ProjectService {
    project_store: Box<dyn ProjectStore>,
}

impl ProjectService {
    #[instrument(skip_all)]
    fn new(project_store: impl ProjectStore + 'static) -> Self {
        let project_service = Self {
            project_store: Box::new(project_store),
        };
        trace!("project service initialized");
        project_service
    }

    #[instrument(skip(self))]
    fn create(&self, title: &str) -> ProjectId {
        let project = Project::new(self.generate_new_ref(), title);
        info!(?project, "new project created");
        let project_id = project.id;
        self.project_store.save(project);
        project_id
    }

    #[instrument(skip(self))]
    fn generate_new_ref(&self) -> String {
        // TODO: generate a random reference that's unique across existing projects
        let new_ref = String::from("ABC");
        debug!(new_ref, "new project reference generated");
        new_ref
    }

    #[instrument(skip(self))]
    fn change_title(&self, id: ProjectId, new_title: &str) -> Result<(), String> {
        let mut project = self
            .project_store
            .get_by_id(id)
            .ok_or("ProjectServiceError::NotFound")?;
        project.title = new_title.to_string();
        info!("project title changed");
        self.project_store.save(project);
        Ok(())
    }

    #[instrument(skip(self))]
    fn get(&self, id: ProjectId) -> Option<Project> {
        self.project_store.get_by_id(id)
    }
}

// --- Infrastructure ---

trait ProjectStore {
    fn get_by_id(&self, id: ProjectId) -> Option<Project>;

    fn save(&self, project: Project);
}

#[derive(Debug, Clone)]
struct ProjectStoreInMemory {
    db: Rc<RefCell<HashMap<ProjectId, Project>>>,
}

impl ProjectStoreInMemory {
    #[instrument]
    fn new() -> Self {
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
    fn save(&self, project: Project) {
        self.db.borrow_mut().insert(project.id, project);
        debug!("project saved");
    }
}

fn main() -> Result<(), String> {
    tracing_subscriber::fmt()
        .pretty()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let project_store = ProjectStoreInMemory::new();
    let project_service = ProjectService::new(project_store);

    // simulation of a scenario
    let project_id = project_service.create("My Project");
    println!(
        "Project created: {}",
        project_service.get(project_id).unwrap()
    );
    project_service.change_title(project_id, "My Project's New Title")?;
    println!(
        "Project title changed: {}",
        project_service.get(project_id).unwrap()
    );

    Ok(())
}
