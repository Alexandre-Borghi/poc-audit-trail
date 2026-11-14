use super::*;

pub struct Missing;

#[must_use]
pub struct RecordBuilder<A = Missing, R = Missing, O = Missing> {
    actor: A,
    resource: R,
    operation: O,
    labels: Labels,
}

impl RecordBuilder<Missing, Missing, Missing> {
    pub fn new() -> Self {
        Self {
            actor: Missing,
            resource: Missing,
            operation: Missing,
            labels: Labels::new(),
        }
    }
}

impl<A, R, O> RecordBuilder<A, R, O> {
    pub fn with_actor(self, kind: String, id: String) -> RecordBuilder<Actor, R, O> {
        RecordBuilder {
            actor: Actor::new(kind, id),
            resource: self.resource,
            operation: self.operation,
            labels: self.labels,
        }
    }

    pub fn with_resource(self, kind: String, id: String) -> RecordBuilder<A, Resource, O> {
        RecordBuilder {
            actor: self.actor,
            resource: Resource::new(kind, id),
            operation: self.operation,
            labels: self.labels,
        }
    }

    pub fn with_operation(
        self,
        kind: OperationKind,
        id: String,
        time: DateTime<Utc>,
        status: OperationStatus,
    ) -> RecordBuilder<A, R, Operation> {
        RecordBuilder {
            actor: self.actor,
            resource: self.resource,
            operation: Operation::new(kind, id, time, status),
            labels: self.labels,
        }
    }

    pub fn with_label(mut self, key: String, value: String) -> Self {
        self.labels.0.insert(key, value);
        self
    }
}

impl<R, O> RecordBuilder<Actor, R, O> {
    pub fn with_actor_metadata(mut self, key: String, value: String) -> Self {
        self.actor.set_metadata(key, value);
        self
    }
}

impl<A, O> RecordBuilder<A, Resource, O> {
    pub fn with_resource_metadata(mut self, key: String, value: String) -> Self {
        self.resource.set_metadata(key, value);
        self
    }

    pub fn try_with_create(
        mut self,
        field: String,
        description: Option<String>,
        new_value: &(impl Serialize + ?Sized),
    ) -> Result<Self, SnapshotError> {
        self.resource
            .try_set_change(field, description, None::<&()>, Some(new_value))?;
        Ok(self)
    }

    pub fn try_with_update(
        mut self,
        field: String,
        description: Option<String>,
        old_value: &(impl Serialize + ?Sized),
        new_value: &(impl Serialize + ?Sized),
    ) -> Result<Self, SnapshotError> {
        self.resource
            .try_set_change(field, description, Some(old_value), Some(new_value))?;
        Ok(self)
    }

    pub fn try_with_delete(
        mut self,
        field: String,
        description: Option<String>,
        old_value: &(impl Serialize + ?Sized),
    ) -> Result<Self, SnapshotError> {
        self.resource
            .try_set_change(field, description, Some(old_value), None::<&()>)?;
        Ok(self)
    }
}

impl<A, R> RecordBuilder<A, R, Operation> {
    pub fn with_operation_metadata(mut self, key: String, value: String) -> Self {
        self.operation.set_metadata(key, value);
        self
    }
}

impl RecordBuilder<Actor, Resource, Operation> {
    pub fn build(self) -> Record {
        Record::new(self.actor, self.resource, self.operation, self.labels)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;

    #[test]
    fn it_works() -> Result<(), Box<dyn Error>> {
        let operation_time = DateTime::from_timestamp_secs(946_684_800).unwrap();
        let _ = RecordBuilder::new()
            .with_actor("user".to_string(), "123".to_string())
            .with_actor_metadata("privilege".to_string(), "admin".to_string())
            .with_resource("order".to_string(), "456".to_string())
            .with_resource_metadata("ref".to_string(), "ABC-DEF".to_string())
            .with_resource_metadata("project_ref".to_string(), "ABC".to_string())
            .try_with_update(
                "status".to_string(),
                Some("automatic status change from carrier API".to_string()),
                "shipped",
                "delivered",
            )?
            .try_with_create("delivery_date".to_string(), None, "2000-01-01T00:00:00Z")?
            .try_with_delete("shipping_status".to_string(), None, "ongoing")?
            .with_operation(
                OperationKind::Update,
                "CarrierTracking".to_string(),
                operation_time,
                OperationStatus::Succeeded,
            )
            .with_operation_metadata("trigger".to_string(), "schedule".to_string())
            .build();
        Ok(())
    }
}
