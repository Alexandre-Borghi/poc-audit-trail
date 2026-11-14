use std::fmt::Display;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod service;
mod store;

pub use service::ProjectService;
pub use store::{ProjectStore, ProjectStoreInMemory};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectId(Uuid);

impl Into<String> for ProjectId {
    fn into(self) -> String {
        self.0.to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    id: ProjectId,
    r#ref: String,
    title: String,
}

impl Project {
    pub fn new(r#ref: String, title: String) -> Self {
        Self {
            id: ProjectId(Uuid::new_v4()),
            r#ref,
            title,
        }
    }

    pub fn id(&self) -> ProjectId {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }
}

impl Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.r#ref, self.title)
    }
}
