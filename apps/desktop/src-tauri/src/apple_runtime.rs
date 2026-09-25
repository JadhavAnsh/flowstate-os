use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::AppHandle;
use tauri_plugin_shell::process::{CommandChild, CommandEvent};
use tauri_plugin_shell::ShellExt;
use tokio::sync::{broadcast, oneshot};
use uuid::Uuid;

const SIDECAR_PATH: &str = "flowstate-apple-runtime";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppleRuntimeStatus {
    pub platform: String,
    pub locale: String,
    pub speech_available: bool,
    pub speech_asset_status: String,
    pub model_available: bool,
    pub model_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppleRuntimeEvent {
    pub id: String,
    pub event: String,
    #[serde(default)]
    pub payload: Value,
    pub code: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AppleRuntimeError {
    pub code: String,
    pub message: String,
}

impl std::fmt::Display for AppleRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for AppleRuntimeError {}

#[derive(Debug, Clone)]
pub struct CaptureContext {
    pub request_id: String,
    pub conversation_id: String,
}

pub struct AppleRuntime {
    app: AppHandle,
    child: Mutex<Option<CommandChild>>,
    pending: Mutex<HashMap<String, oneshot::Sender<Result<Value, AppleRuntimeError>>>>,
    events: broadcast::Sender<AppleRuntimeEvent>,
    active_capture: Mutex<Option<CaptureContext>>,
    restart_budget: AtomicUsize,
}

impl AppleRuntime {
    pub fn start(app: AppHandle) -> Arc<Self> {
        let (events, _) = broadcast::channel(256);
        let runtime = Arc::new(Self {
            app,
            child: Mutex::new(None),
            pending: Mutex::new(HashMap::new()),
            events,
            active_capture: Mutex::new(None),
            restart_budget: AtomicUsize::new(1),
        });
        if let Err(error) = runtime.spawn_process() {
            eprintln!("Apple runtime did not start: {error}");
        }
        runtime
    }

    pub fn subscribe(&self) -> broadcast::Receiver<AppleRuntimeEvent> {
        self.events.subscribe()
    }

    pub fn capture_context(&self) -> Option<CaptureContext> {
        self.active_capture.lock().expect("capture lock").clone()
    }

    fn spawn_process(self: &Arc<Self>) -> Result<(), AppleRuntimeError> {
        let (mut receiver, child) = self
            .app
            .shell()
            .sidecar(SIDECAR_PATH)
            .map_err(|error| AppleRuntimeError {
                code: "apple_runtime_spawn".to_string(),
                message: error.to_string(),
            })?
            .spawn()
            .map_err(|error| AppleRuntimeError {
                code: "apple_runtime_spawn".to_string(),
                message: error.to_string(),
            })?;
        *self.child.lock().expect("child lock") = Some(child);

        let runtime = Arc::clone(self);
        tauri::async_runtime::spawn(async move {
            while let Some(event) = receiver.recv().await {
                match event {
                    CommandEvent::Stdout(line) => runtime.handle_stdout(&line),
                    CommandEvent::Stderr(line) => {
                        let message = String::from_utf8_lossy(&line);
                        eprintln!("Apple runtime: {message}");
                    }
                    CommandEvent::Error(message) => {
                        runtime.fail_all("apple_runtime_io", &message);
                    }
                    CommandEvent::Terminated(status) => {
                        *runtime.child.lock().expect("child lock") = None;
                        runtime.fail_all(
                            "apple_runtime_terminated",
                            &format!("Apple runtime exited with {:?}", status.code),
                        );
                        break;
                    }
                    _ => {}
                }
            }
        });
        Ok(())
    }

    fn ensure_started(self: &Arc<Self>) -> Result<(), AppleRuntimeError> {
        if self.child.lock().expect("child lock").is_some() {
            return Ok(());
        }
        if self
            .restart_budget
            .compare_exchange(1, 0, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Err(AppleRuntimeError {
                code: "apple_runtime_unavailable".to_string(),
                message: "Apple runtime stopped and its automatic restart was already used."
                    .to_string(),
            });
        }
        self.spawn_process()
    }

    fn handle_stdout(&self, line: &[u8]) {
        let event: AppleRuntimeEvent = match serde_json::from_slice(line) {
            Ok(event) => event,
            Err(error) => {
                eprintln!("Apple runtime emitted invalid JSON: {error}");
                return;
            }
        };
        let _ = self.events.send(event.clone());
        if event.event != "result" && event.event != "error" {
            return;
        }
        let sender = self.pending.lock().expect("pending lock").remove(&event.id);
        if let Some(sender) = sender {
            let result = if event.event == "result" {
                Ok(event.payload)
            } else {
                Err(AppleRuntimeError {
                    code: event
                        .code
                        .unwrap_or_else(|| "apple_runtime_error".to_string()),
                    message: event
                        .message
                        .unwrap_or_else(|| "Apple runtime failed.".to_string()),
                })
            };
            let _ = sender.send(result);
        }
    }

    fn fail_all(&self, code: &str, message: &str) {
        let pending = std::mem::take(&mut *self.pending.lock().expect("pending lock"));
        for (_, sender) in pending {
            let _ = sender.send(Err(AppleRuntimeError {
                code: code.to_string(),
                message: message.to_string(),
            }));
        }
    }

    async fn request(
        self: &Arc<Self>,
        command: &str,
        fields: Value,
        timeout: Duration,
    ) -> Result<(String, Value), AppleRuntimeError> {
        let id = Uuid::new_v4().to_string();
        let payload = self.request_with_id(&id, command, fields, timeout).await?;
        Ok((id, payload))
    }

    async fn request_with_id(
        self: &Arc<Self>,
        id: &str,
        command: &str,
        fields: Value,
        timeout: Duration,
    ) -> Result<Value, AppleRuntimeError> {
        self.ensure_started()?;
        let mut body = fields.as_object().cloned().unwrap_or_default();
        body.insert("id".to_string(), Value::String(id.to_string()));
        body.insert("command".to_string(), Value::String(command.to_string()));
        let mut encoded =
            serde_json::to_vec(&Value::Object(body)).map_err(|error| AppleRuntimeError {
                code: "apple_runtime_encode".to_string(),
                message: error.to_string(),
            })?;
        encoded.push(b'\n');

        let (sender, receiver) = oneshot::channel();
        self.pending
            .lock()
            .expect("pending lock")
            .insert(id.to_string(), sender);
        let write_result = self
            .child
            .lock()
            .expect("child lock")
            .as_mut()
            .ok_or_else(|| AppleRuntimeError {
                code: "apple_runtime_unavailable".to_string(),
                message: "Apple runtime is not running.".to_string(),
            })?
            .write(&encoded);
        if let Err(error) = write_result {
            self.pending.lock().expect("pending lock").remove(id);
            return Err(AppleRuntimeError {
                code: "apple_runtime_io".to_string(),
                message: error.to_string(),
            });
        }

        let result = match tokio::time::timeout(timeout, receiver).await {
            Ok(result) => result,
            Err(_) => {
                self.pending.lock().expect("pending lock").remove(id);
                return Err(AppleRuntimeError {
                    code: "apple_runtime_timeout".to_string(),
                    message: format!("Apple runtime command {command} timed out."),
                });
            }
        }
        .map_err(|_| AppleRuntimeError {
            code: "apple_runtime_stopped".to_string(),
            message: "Apple runtime stopped before replying.".to_string(),
        })??;
        Ok(result)
    }

    pub async fn status(self: &Arc<Self>) -> Result<AppleRuntimeStatus, AppleRuntimeError> {
        let (_, payload) = self
            .request(
                "status",
                json!({ "locale": "en-IN" }),
                Duration::from_secs(15),
            )
            .await?;
        serde_json::from_value(payload).map_err(|error| AppleRuntimeError {
            code: "apple_runtime_decode".to_string(),
            message: error.to_string(),
        })
    }

    pub async fn install_speech_assets(self: &Arc<Self>) -> Result<Value, AppleRuntimeError> {
        let (_, payload) = self
            .request(
                "install_speech_assets",
                json!({ "locale": "en-IN" }),
                Duration::from_secs(900),
            )
            .await?;
        Ok(payload)
    }

    pub async fn start_capture(
        self: &Arc<Self>,
        conversation_id: String,
    ) -> Result<String, AppleRuntimeError> {
        self.cancel_speech().await?;
        let _ = self.cancel_generation(&conversation_id).await;
        let (request_id, _) = self
            .request(
                "start_capture",
                json!({ "locale": "en-IN" }),
                Duration::from_secs(30),
            )
            .await?;
        *self.active_capture.lock().expect("capture lock") = Some(CaptureContext {
            request_id: request_id.clone(),
            conversation_id,
        });
        Ok(request_id)
    }

    pub async fn stop_capture(self: &Arc<Self>) -> Result<String, AppleRuntimeError> {
        let (_, payload) = self
            .request("stop_capture", json!({}), Duration::from_secs(30))
            .await?;
        *self.active_capture.lock().expect("capture lock") = None;
        Ok(payload
            .get("text")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string())
    }

    pub async fn cancel_capture(self: &Arc<Self>) -> Result<(), AppleRuntimeError> {
        let result = self
            .request("cancel_capture", json!({}), Duration::from_secs(10))
            .await
            .map(|_| ());
        *self.active_capture.lock().expect("capture lock") = None;
        result
    }

    pub async fn assess(self: &Arc<Self>, prompt: &str) -> Result<Value, AppleRuntimeError> {
        let (_, payload) = self
            .request(
                "assess",
                json!({ "prompt": prompt }),
                Duration::from_secs(30),
            )
            .await?;
        Ok(payload)
    }

    pub async fn generate_stream(
        self: &Arc<Self>,
        conversation_id: &str,
        prompt: &str,
        instructions: &str,
        on_delta: impl Fn(String) -> Result<(), AppleRuntimeError>,
    ) -> Result<String, AppleRuntimeError> {
        let id = Uuid::new_v4().to_string();
        let mut events = self.subscribe();
        let request = self.request_with_id(
            &id,
            "generate",
            json!({
                "conversationId": conversation_id,
                "prompt": prompt,
                "instructions": instructions,
            }),
            Duration::from_secs(180),
        );
        tokio::pin!(request);
        loop {
            tokio::select! {
                result = &mut request => {
                    let payload = result?;
                    return Ok(payload.get("text").and_then(Value::as_str).unwrap_or_default().to_string());
                }
                event = events.recv() => {
                    match event {
                        Ok(event) if event.id == id && event.event == "model_snapshot" => {
                            if let Some(delta) = event.payload.get("delta").and_then(Value::as_str) {
                                on_delta(delta.to_string())?;
                            }
                        }
                        Ok(_) | Err(broadcast::error::RecvError::Lagged(_)) => {}
                        Err(broadcast::error::RecvError::Closed) => {
                            return Err(AppleRuntimeError { code: "apple_runtime_stopped".to_string(), message: "Apple runtime event channel closed.".to_string() });
                        }
                    }
                }
            }
        }
    }

    pub async fn cancel_generation(
        self: &Arc<Self>,
        conversation_id: &str,
    ) -> Result<(), AppleRuntimeError> {
        self.request(
            "cancel_generation",
            json!({ "conversationId": conversation_id }),
            Duration::from_secs(10),
        )
        .await
        .map(|_| ())
    }

    pub async fn speak(self: &Arc<Self>, text: &str) -> Result<(), AppleRuntimeError> {
        self.request(
            "speak",
            json!({ "text": text, "locale": "en-IN" }),
            Duration::from_secs(10),
        )
        .await
        .map(|_| ())
    }

    pub async fn cancel_speech(self: &Arc<Self>) -> Result<(), AppleRuntimeError> {
        self.request("cancel_speech", json!({}), Duration::from_secs(10))
            .await
            .map(|_| ())
    }

    pub fn shutdown(self: &Arc<Self>) {
        let id = Uuid::new_v4().to_string();
        let encoded = format!("{{\"id\":\"{id}\",\"command\":\"shutdown\"}}\n");
        if let Some(child) = self.child.lock().expect("child lock").as_mut() {
            let _ = child.write(encoded.as_bytes());
        }
    }
}
