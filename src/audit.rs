use std::{collections::BTreeMap, fmt::Display};

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::value::RawValue;
use thiserror::Error;
use uuid::Uuid;

mod builder;
mod service;
mod store;

pub use builder::RecordBuilder;
pub use service::AuditService;
pub use store::AuditStoreInMemory;

use store::AuditStore;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct RecordId(Uuid);

impl Display for RecordId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Record {
    id: RecordId,
    creation_time: DateTime<Utc>,
    labels: Labels,
    actor: Actor,
    resource: Resource,
    operation: Operation,
}

impl Record {
    fn new(actor: Actor, resource: Resource, operation: Operation, labels: Labels) -> Self {
        Self {
            id: RecordId(Uuid::new_v4()),
            creation_time: Utc::now(),
            labels,
            actor,
            resource,
            operation,
        }
    }
}

impl Display for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} | {}", self.id, self.creation_time)?;
        writeln!(f, "  - Actor: {} (id: {})", self.actor.kind, self.actor.id)?;
        writeln!(
            f,
            "  - Operation: {} (kind: {}) | {} | {:?}",
            self.operation.id, self.operation.kind, self.operation.time, self.operation.status
        )?;
        writeln!(
            f,
            "  - Resource: {} (id: {})",
            self.resource.kind, self.resource.id
        )?;
        for (field, change) in self.resource.changes.iter() {
            write!(f, "    - {field}: ")?;
            if let Some(ref old_value) = change.old_value {
                write!(f, "{}", old_value.0)?;
            } else {
                write!(f, "nothing")?;
            }
            write!(f, " => ")?;
            if let Some(ref new_value) = change.new_value {
                write!(f, "{}", new_value.0)?;
            } else {
                write!(f, "nothing")?;
            }
            if let Some(ref description) = change.description {
                write!(f, "{description}")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct Labels(BTreeMap<String, String>);

impl Labels {
    fn new() -> Self {
        Self(BTreeMap::new())
    }
}

#[derive(Debug, Clone)]
pub struct Actor {
    kind: String,
    id: String,
    metadata: Metadata,
}

impl Actor {
    pub fn new(kind: String, id: String) -> Self {
        Self {
            kind,
            id,
            metadata: Metadata::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.0.insert(key, value);
        self
    }
}

#[derive(Debug, Clone)]
pub struct Resource {
    kind: String,
    id: String,
    metadata: Metadata,
    changes: BTreeMap<String, ResourceChange>,
}

impl Resource {
    pub fn new(kind: String, id: String) -> Self {
        Self {
            kind,
            id,
            metadata: Metadata::new(),
            changes: BTreeMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.0.insert(key, value);
        self
    }

    pub fn try_with_change(
        mut self,
        name: String,
        description: Option<String>,
        old_value: Option<&(impl Serialize + ?Sized)>,
        new_value: Option<&(impl Serialize + ?Sized)>,
    ) -> Result<Self, SnapshotError> {
        self.changes.insert(
            name,
            ResourceChange {
                description,
                old_value: old_value.map(Snapshot::new).transpose()?,
                new_value: new_value.map(Snapshot::new).transpose()?,
            },
        );
        Ok(self)
    }
}

#[derive(Debug, Clone)]
struct ResourceChange {
    description: Option<String>,
    old_value: Option<Snapshot>,
    new_value: Option<Snapshot>,
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SnapshotError {
    #[error("failed to serialize snapshot: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
struct Snapshot(Box<RawValue>);

impl Snapshot {
    pub fn new(value: impl Serialize) -> Result<Self, SnapshotError> {
        let json = serde_json::to_string(&value)?;
        Ok(Self(RawValue::from_string(json)?))
    }
}

#[derive(Debug, Clone)]
pub struct Operation {
    kind: String,
    id: String,
    time: DateTime<Utc>,
    metadata: Metadata,
    status: Option<OperationStatus>,
}

#[derive(Debug, Clone)]
pub enum OperationStatus {
    Succeeded,
    Failed,
    Denied,
}

impl Operation {
    pub fn new(
        kind: String,
        id: String,
        time: DateTime<Utc>,
        status: Option<OperationStatus>,
    ) -> Self {
        Self {
            kind,
            id,
            time,
            metadata: Metadata::new(),
            status,
        }
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.0.insert(key, value);
        self
    }
}

#[derive(Debug, Clone)]
struct Metadata(BTreeMap<String, String>);

impl Metadata {
    fn new() -> Self {
        Self(BTreeMap::new())
    }
}

// TODO: builder pattern constructor for Record
