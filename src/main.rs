use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{Debug, Display},
    rc::Rc,
};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument, trace};
use uuid::Uuid;

mod audit;

use audit::{AuditService, AuditStoreInMemory};

use crate::audit::RecordBuilder;

// --- Domain ---

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
struct ProjectId(Uuid);

impl Into<String> for ProjectId {
    fn into(self) -> String {
        self.0.to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Project {
    id: ProjectId,
    r#ref: String,
    title: String,
}

impl Project {
    fn new(r#ref: String, title: String) -> Self {
        Self {
            id: ProjectId(Uuid::new_v4()),
            r#ref,
            title,
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
    audit_service: AuditService,
}

impl ProjectService {
    #[instrument(skip_all)]
    fn new(project_store: impl ProjectStore + 'static, audit_service: AuditService) -> Self {
        let project_service = Self {
            project_store: Box::new(project_store),
            audit_service,
        };
        trace!("project service initialized");
        project_service
    }

    #[instrument(skip(self))]
    fn create(&self, title: String) -> ProjectId {
        let project = Project::new(self.generate_new_ref(), title);
        info!(?project, "new project created");
        self.audit_service.record(
            &RecordBuilder::new()
                .with_actor(audit::Actor::new("system".to_string(), "".to_string()))
                .with_resource(
                    audit::Resource::new("project".to_string(), project.id.into())
                        .try_with_change("self".to_string(), None, None::<&()>, Some(&project))
                        .unwrap(),
                )
                .with_operation(audit::Operation::new(
                    "create".to_string(),
                    "ProjectService::create".to_string(),
                    Utc::now(),
                    Some(audit::OperationStatus::Succeeded),
                ))
                .build(),
        );
        self.project_store.save(&project);
        project.id
    }

    #[instrument(skip(self))]
    fn generate_new_ref(&self) -> String {
        // TODO: generate a random reference that's unique across existing projects
        let new_ref = String::from("ABC");
        debug!(new_ref, "new project reference generated");
        new_ref
    }

    #[instrument(skip(self))]
    fn change_title(&self, id: ProjectId, new_title: String) -> Result<(), String> {
        let mut project = self
            .project_store
            .get_by_id(id)
            .ok_or("ProjectServiceError::NotFound")?;
        let old_title = project.title;
        project.title = new_title.clone();
        info!("project title changed");
        self.audit_service.record(
            &RecordBuilder::new()
                .with_actor(audit::Actor::new("system".to_string(), "".to_string()))
                .with_resource(
                    audit::Resource::new("project".to_string(), project.id.into())
                        .try_with_change(
                            "title".to_string(),
                            None,
                            Some(&old_title),
                            Some(&new_title),
                        )
                        .unwrap(),
                )
                .with_operation(audit::Operation::new(
                    "update".to_string(),
                    "ProjectService::change_title".to_string(),
                    Utc::now(),
                    Some(audit::OperationStatus::Succeeded),
                ))
                .build(),
        );
        self.project_store.save(&project);
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

    fn save(&self, project: &Project);
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
    fn save(&self, project: &Project) {
        self.db.borrow_mut().insert(project.id, project.clone());
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

    let audit_store = AuditStoreInMemory::new();
    let audit_service = AuditService::new(audit_store.clone());

    let project_store = ProjectStoreInMemory::new();
    let project_service = ProjectService::new(project_store.clone(), audit_service.clone());

    // simulation of a scenario
    let project_id = project_service.create("My Project".to_string());
    println!(
        "Project created: {}",
        project_service.get(project_id).unwrap()
    );
    project_service.change_title(project_id, "My Project's New Title".to_string())?;
    println!(
        "Project title changed: {}",
        project_service.get(project_id).unwrap()
    );

    let audit_records = audit_service.list_all();
    for record in audit_records {
        println!("{record}");
    }

    Ok(())
}
