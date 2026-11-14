mod audit;
mod project;

use crate::{
    audit::{AuditService, AuditStoreInMemory},
    project::{ProjectService, ProjectStoreInMemory},
};

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
