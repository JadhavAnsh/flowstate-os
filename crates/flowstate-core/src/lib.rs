mod core;
mod error;
mod events;
mod persistence;
mod providers;
mod vault;

pub use core::{
    CoreHealth, FlowStateCore, ProviderPublicConfig, ProviderStatus, SendMessageOutcome,
    SessionSnapshot, CORE_VERSION, DEFAULT_PROVIDER_ID,
};
pub use error::CoreError;
pub use flowstate_protocol::Event;
pub use persistence::{ConversationRow, MessageRow, RunRow, TaskRow};
pub use providers::{
    DeltaSink, ModelRequest, ModelRoute, RouteAssessment, SpeechInputProvider,
    SpeechOutputProvider, TextModelProvider,
};
