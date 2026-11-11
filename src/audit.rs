use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::value::RawValue;
use thiserror::Error;
use uuid::Uuid;

mod builder;

pub use builder::RecordBuilder;

struct Record {
    id: Uuid,
    creation_time: DateTime<Utc>,
    labels: Labels,
    actor: Actor,
    resource: Resource,
    operation: Operation,
}

impl Record {
    fn new(actor: Actor, resource: Resource, operation: Operation, labels: Labels) -> Self {
        Self {
            id: Uuid::new_v4(),
            creation_time: Utc::now(),
            labels,
            actor,
            resource,
            operation,
        }
    }
}

struct Labels(BTreeMap<String, String>);

impl Labels {
    fn new() -> Self {
        Self(BTreeMap::new())
    }
}

struct Actor {
    kind: String,
    id: String,
    metadata: Metadata,
}

impl Actor {
    fn new(kind: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            id: id.into(),
            metadata: Metadata::new(),
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.0.insert(key.into(), value.into());
        self
    }
}

struct Resource {
    kind: String,
    id: String,
    metadata: Metadata,
    changes: BTreeMap<String, ResourceChange>,
}

impl Resource {
    fn new(kind: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            id: id.into(),
            metadata: Metadata::new(),
            changes: BTreeMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.0.insert(key.into(), value.into());
        self
    }

    pub fn try_with_change(
        mut self,
        name: impl Into<String>,
        description: Option<impl Into<String>>,
        old_value: Option<impl Serialize>,
        new_value: Option<impl Serialize>,
    ) -> Result<Self, SnapshotError> {
        self.changes.insert(
            name.into(),
            ResourceChange {
                description: description.map(Into::into),
                old_value: old_value.map(Snapshot::new).transpose()?,
                new_value: new_value.map(Snapshot::new).transpose()?,
            },
        );
        Ok(self)
    }
}

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

struct Snapshot(Box<RawValue>);

impl Snapshot {
    pub fn new(value: impl Serialize) -> Result<Self, SnapshotError> {
        let json = serde_json::to_string(&value)?;
        Ok(Self(RawValue::from_string(json)?))
    }
}
struct Operation {
    kind: String,
    id: String,
    time: DateTime<Utc>,
    metadata: Metadata,
    status: Option<OperationStatus>,
}

enum OperationStatus {
    Succeeded,
    Failed,
    Denied,
}

impl Operation {
    fn new(
        kind: impl Into<String>,
        id: impl Into<String>,
        time: DateTime<Utc>,
        status: Option<OperationStatus>,
    ) -> Self {
        Self {
            kind: kind.into(),
            id: id.into(),
            time,
            metadata: Metadata::new(),
            status,
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.0.insert(key.into(), value.into());
        self
    }
}

struct Metadata(BTreeMap<String, String>);

impl Metadata {
    fn new() -> Self {
        Self(BTreeMap::new())
    }
}

// TODO: builder pattern constructor for Record
