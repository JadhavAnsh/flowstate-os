use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::error::{CoreError, CoreResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

pub async fn stream_openai_chat(
    api_key: &str,
    model: &str,
    messages: &[ChatMessage],
    cancel: Arc<AtomicBool>,
    mut on_delta: impl FnMut(String) -> CoreResult<()>,
) -> CoreResult<String> {
    let client = Client::new();
    let body = json!({
        "model": model,
        "stream": true,
        "messages": messages.iter().map(|m| json!({"role": m.role, "content": m.content})).collect::<Vec<_>>(),
    });

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(CoreError::Provider(format!("OpenAI HTTP {status}: {text}")));
    }

    let mut stream = response.bytes_stream();
    let mut assembled = String::new();
    let mut buffer = String::new();

    while let Some(chunk) = stream.next().await {
        if cancel.load(Ordering::Relaxed) {
            return Err(CoreError::Cancelled);
        }
        let chunk = chunk?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(pos) = buffer.find("\n\n") {
            let frame = buffer.drain(..=pos).collect::<String>();
            for line in frame.lines() {
                let line = line.trim();
                if !line.starts_with("data: ") {
                    continue;
                }
                let data = &line[6..];
                if data == "[DONE]" {
                    continue;
                }
                let parsed: StreamChunk = serde_json::from_str(data)
                    .map_err(|e| CoreError::Provider(format!("invalid stream chunk: {e}")))?;
                if let Some(choice) = parsed.choices.first() {
                    if let Some(delta) = &choice.delta.content {
                        assembled.push_str(delta);
                        on_delta(delta.clone())?;
                    }
                }
            }
        }
    }

    Ok(assembled)
}

#[derive(Debug, Deserialize)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: StreamDelta,
}

#[derive(Debug, Deserialize)]
struct StreamDelta {
    content: Option<String>,
}
