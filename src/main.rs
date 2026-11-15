use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post, put},
};
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use tower_http::trace::TraceLayer;

use crate::{
    audit::{AuditService, AuditStoreInMemory},
    project::{ProjectService, ProjectStoreInMemory},
};

pub mod audit;
pub mod project;

pub struct AppState {
    project_service: ProjectService,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = init_tracing_opentelemetry::TracingConfig::development().init_subscriber()?;

    let audit_store = AuditStoreInMemory::new();
    let audit_service = AuditService::new(audit_store.clone());

    let project_store = ProjectStoreInMemory::new();
    let project_service = ProjectService::new(project_store.clone(), audit_service.clone());

    let state = Arc::new(AppState { project_service });

    let app = Router::new()
        .route("/projects", post(api::project::create))
        .route("/projects/{id}", get(api::project::get))
        .route("/projects/{id}/title", put(api::project::change_title))
        .layer(TraceLayer::new_for_http())
        .layer(OtelInResponseLayer)
        .layer(OtelAxumLayer::default())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

pub mod api {
    pub mod project {
        use std::sync::Arc;

        use axum::{
            Json,
            extract::{Path, State},
            http::StatusCode,
        };

        use crate::{
            AppState,
            project::{Project, ProjectId},
        };

        pub async fn create() {}

        pub async fn get(
            Path(id): Path<ProjectId>,
            State(state): State<Arc<AppState>>,
        ) -> Result<Json<Project>, StatusCode> {
            state
                .project_service
                .get(id)
                .map(Json)
                .ok_or(StatusCode::NOT_FOUND)
        }

        pub async fn change_title() {}
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        audit::{AuditService, AuditStoreInMemory},
        project::{ProjectService, ProjectStoreInMemory},
    };

    #[test]
    fn it_works() -> Result<(), String> {
        let audit_store = AuditStoreInMemory::new();
        let audit_service = AuditService::new(audit_store.clone());

        let project_store = ProjectStoreInMemory::new();
        let project_service = ProjectService::new(project_store.clone(), audit_service.clone());

        let project_id = project_service.create("My Project".to_string());
        let project = project_service.get(project_id).unwrap();
        assert_eq!(project.title(), "My Project");
        project_service.change_title(project_id, "My Project's New Title".to_string())?;
        let project = project_service.get(project_id).unwrap();
        assert_eq!(project.title(), "My Project's New Title");

        Ok(())
    }
}
