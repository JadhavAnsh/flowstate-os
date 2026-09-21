use flowstate_protocol::Event;

use crate::error::{CoreError, CoreResult};

pub const SUPPORTED_SCHEMA_VERSION: i64 = 1;

pub fn validate_event(event: &Event) -> CoreResult<()> {
    if event.schema_version != SUPPORTED_SCHEMA_VERSION {
        return Err(CoreError::UnsupportedSchemaVersion(event.schema_version));
    }
    Ok(())
}
