use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use flowstate_core::{
    CoreError, DeltaSink, ModelRequest, ModelRoute, RouteAssessment, SpeechInputProvider,
    SpeechOutputProvider, TextModelProvider,
};

use crate::apple_runtime::AppleRuntime;

pub struct AppleFoundationProvider {
    runtime: Arc<AppleRuntime>,
}

pub struct AppleSpeechProvider;

#[async_trait]
impl SpeechInputProvider for AppleSpeechProvider {
    fn provider_id(&self) -> &'static str {
        "apple-speech"
    }
}

pub struct AppleSpeechOutputProvider;

#[async_trait]
impl SpeechOutputProvider for AppleSpeechOutputProvider {
    fn provider_id(&self) -> &'static str {
        "apple-speech"
    }
}

impl AppleFoundationProvider {
    pub fn new(runtime: Arc<AppleRuntime>) -> Self {
        Self { runtime }
    }
}

#[async_trait]
impl TextModelProvider for AppleFoundationProvider {
    fn provider_id(&self) -> &'static str {
        "apple-foundation"
    }

    fn model_name(&self) -> &'static str {
        "system"
    }

    async fn assess(&self, prompt: &str) -> Result<RouteAssessment, CoreError> {
        let payload = self
            .runtime
            .assess(prompt)
            .await
            .map_err(|error| CoreError::Provider(error.to_string()))?;
        let route = match payload.get("route").and_then(serde_json::Value::as_str) {
            Some("local") => ModelRoute::Local,
            _ => ModelRoute::Cloud,
        };
        Ok(RouteAssessment {
            route,
            reason_code: payload
                .get("reasonCode")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("cloud_recommended")
                .to_string(),
        })
    }

    async fn stream(
        &self,
        request: ModelRequest,
        cancel: Arc<AtomicBool>,
        on_delta: DeltaSink,
    ) -> Result<String, CoreError> {
        let instructions = "You are FlowState, a concise local desktop assistant. Use only the user's supplied context and stable general knowledge. State limitations plainly. Never claim to have browsed the web.";
        self.runtime
            .generate_stream(
                &request.conversation_id,
                &request.prompt,
                instructions,
                |delta| {
                    if cancel.load(Ordering::Relaxed) {
                        return Err(crate::apple_runtime::AppleRuntimeError {
                            code: "cancelled".to_string(),
                            message: "Generation cancelled.".to_string(),
                        });
                    }
                    on_delta(delta).map_err(|error| crate::apple_runtime::AppleRuntimeError {
                        code: "event_error".to_string(),
                        message: error.to_string(),
                    })
                },
            )
            .await
            .map_err(|error| {
                if error.code == "cancelled" {
                    CoreError::Cancelled
                } else {
                    CoreError::Provider(error.to_string())
                }
            })
    }

    async fn cancel(&self, conversation_id: &str) -> Result<(), CoreError> {
        self.runtime
            .cancel_generation(conversation_id)
            .await
            .map_err(|error| CoreError::Provider(error.to_string()))
    }
}
