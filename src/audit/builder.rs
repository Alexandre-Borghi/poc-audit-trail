use super::*;

struct Missing;

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
    pub fn with_actor(self, actor: Actor) -> RecordBuilder<Actor, R, O> {
        RecordBuilder {
            actor,
            resource: self.resource,
            operation: self.operation,
            labels: self.labels,
        }
    }

    pub fn with_resource(self, resource: Resource) -> RecordBuilder<A, Resource, O> {
        RecordBuilder {
            actor: self.actor,
            resource,
            operation: self.operation,
            labels: self.labels,
        }
    }

    pub fn with_operation(self, operation: Operation) -> RecordBuilder<A, R, Operation> {
        RecordBuilder {
            actor: self.actor,
            resource: self.resource,
            operation,
            labels: self.labels,
        }
    }

    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.0.insert(key.into(), value.into());
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
        let new_record = RecordBuilder::new()
            .with_actor(Actor::new("user", "123").with_metadata("privilege", "admin"))
            .with_resource(
                Resource::new("order", "456")
                    .with_metadata("ref", "ABC-DEF")
                    .with_metadata("project_ref", "ABC")
                    .try_with_change(
                        "status",
                        Some("automatic status change from carrier API"),
                        Some("shipped"),
                        Some("delivered"),
                    )?
                    .try_with_change(
                        "delivery_date",
                        None::<String>,
                        None::<()>,
                        Some("2000-01-01T00:00:00Z"),
                    )?,
            )
            .with_operation(
                Operation::new(
                    "update",
                    "CarrierTracking",
                    operation_time,
                    Some(OperationStatus::Succeeded),
                )
                .with_metadata("trigger", "schedule"),
            )
            .build();
        Ok(())
    }
}
