use chrono::Utc;
use tracing::{debug, info, instrument, trace};

use crate::{
    audit::{self, AuditService, RecordBuilder},
    project::{Project, ProjectId, ProjectStore},
};

pub struct ProjectService {
    project_store: Box<dyn ProjectStore>,
    audit_service: AuditService,
}

impl ProjectService {
    #[instrument(skip_all)]
    pub fn new(project_store: impl ProjectStore + 'static, audit_service: AuditService) -> Self {
        let project_service = Self {
            project_store: Box::new(project_store),
            audit_service,
        };
        trace!("project service initialized");
        project_service
    }

    #[instrument(skip(self))]
    pub fn create(&self, title: String) -> ProjectId {
        let project = Project::new(self.generate_new_ref(), title);
        info!(?project, "new project created");
        self.audit_service.record(
            &RecordBuilder::new()
                .with_actor("system".to_string(), "".to_string())
                .with_resource("project".to_string(), project.id().into())
                .try_with_create("self".to_string(), None, &project)
                .unwrap()
                .with_operation(
                    "create".to_string(),
                    "ProjectService::create".to_string(),
                    Utc::now(),
                    Some(audit::OperationStatus::Succeeded),
                )
                .build(),
        );
        self.project_store.save(&project);
        project.id()
    }

    #[instrument(skip(self))]
    fn generate_new_ref(&self) -> String {
        // TODO: generate a random reference that's unique across existing projects
        let new_ref = String::from("ABC");
        debug!(new_ref, "new project reference generated");
        new_ref
    }

    #[instrument(skip(self))]
    pub fn change_title(&self, id: ProjectId, new_title: String) -> Result<(), String> {
        let mut project = self
            .project_store
            .get_by_id(id)
            .ok_or("ProjectServiceError::NotFound")?;
        let old_title = project.title().to_string();
        project.set_title(new_title.clone());
        info!("project title changed");
        self.audit_service.record(
            &RecordBuilder::new()
                .with_actor("system".to_string(), "".to_string())
                .with_resource("project".to_string(), project.id().into())
                .try_with_update("title".to_string(), None, &old_title, &new_title)
                .unwrap()
                .with_operation(
                    "update".to_string(),
                    "ProjectService::change_title".to_string(),
                    Utc::now(),
                    Some(audit::OperationStatus::Succeeded),
                )
                .build(),
        );
        self.project_store.save(&project);
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn get(&self, id: ProjectId) -> Option<Project> {
        self.project_store.get_by_id(id)
    }
}
