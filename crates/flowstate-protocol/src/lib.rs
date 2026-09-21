//! Types generated at compile time from the canonical JSON Schema.

typify::import_types!(schema = "../../packages/protocol/schemas/protocol.schema.json");

#[cfg(test)]
mod tests {
    use super::{Event, ProtocolRecord};

    #[test]
    fn all_shared_fixtures_deserialize() {
        let fixtures = [
            include_str!("../../../packages/protocol/fixtures/Agent.json"),
            include_str!("../../../packages/protocol/fixtures/Task.json"),
            include_str!("../../../packages/protocol/fixtures/Run.json"),
            include_str!("../../../packages/protocol/fixtures/Tool.json"),
            include_str!("../../../packages/protocol/fixtures/Skill.json"),
            include_str!("../../../packages/protocol/fixtures/Workflow.json"),
            include_str!("../../../packages/protocol/fixtures/Artifact.json"),
            include_str!("../../../packages/protocol/fixtures/Event.json"),
            include_str!("../../../packages/protocol/fixtures/Model.json"),
            include_str!("../../../packages/protocol/fixtures/Permission.json"),
            include_str!("../../../packages/protocol/fixtures/Memory.json"),
            include_str!("../../../packages/protocol/fixtures/Device.json"),
        ];

        for fixture in fixtures {
            let _: ProtocolRecord = serde_json::from_str(fixture).expect("shared fixture");
        }
    }

    #[test]
    fn event_round_trips() {
        let fixture = include_str!("../../../packages/protocol/fixtures/Event.json");
        let event: Event = serde_json::from_str(fixture).expect("valid event fixture");
        let serialized = serde_json::to_string(&event).expect("serialize event");
        let _: Event = serde_json::from_str(&serialized).expect("round trip event");
    }
}
