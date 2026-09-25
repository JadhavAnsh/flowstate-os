mod openai;

pub use openai::{stream_openai_chat, ChatMessage};

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::CoreResult;

pub type DeltaSink = Arc<dyn Fn(String) -> CoreResult<()> + Send + Sync>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ModelRoute {
    Local,
    Cloud,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteAssessment {
    pub route: ModelRoute,
    pub reason_code: String,
}

#[derive(Debug, Clone)]
pub struct ModelRequest {
    pub conversation_id: String,
    pub prompt: String,
    pub history: Vec<ChatMessage>,
}

#[async_trait]
pub trait TextModelProvider: Send + Sync {
    fn provider_id(&self) -> &'static str;
    fn model_name(&self) -> &'static str;
    async fn assess(&self, prompt: &str) -> CoreResult<RouteAssessment>;
    async fn stream(
        &self,
        request: ModelRequest,
        cancel: Arc<AtomicBool>,
        on_delta: DeltaSink,
    ) -> CoreResult<String>;
    async fn cancel(&self, _conversation_id: &str) -> CoreResult<()> {
        Ok(())
    }
}

#[async_trait]
pub trait SpeechInputProvider: Send + Sync {
    fn provider_id(&self) -> &'static str;
}

#[async_trait]
pub trait SpeechOutputProvider: Send + Sync {
    fn provider_id(&self) -> &'static str;
}
