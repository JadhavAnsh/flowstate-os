use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use chrono::Utc;
use flowstate_protocol::Event;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::{CoreError, CoreResult};
use crate::events::{validate_event, EventBus, SUPPORTED_SCHEMA_VERSION};
use crate::persistence::{
    ConversationRow, Database, MessageRow, PendingPermissionRow, ProviderConfigRow, RunRow, TaskRow,
};
use crate::providers::{
    stream_openai_chat, ChatMessage, DeltaSink, ModelRequest, ModelRoute, TextModelProvider,
};
use crate::vault::CredentialVault;

pub const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DEFAULT_PROVIDER_ID: &str = "openai";
pub const DEFAULT_OPENAI_MODEL: &str = "gpt-4o-mini";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreHealth {
    pub status: String,
    pub version: String,
    pub protocol_version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStatus {
    pub provider_id: String,
    pub connected: bool,
    pub default_model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPublicConfig {
    pub provider_id: String,
    pub default_model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub active_conversation_id: Option<String>,
    pub active_task: Option<TaskRow>,
    pub active_run: Option<RunRow>,
    pub voice_listening: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageOutcome {
    pub status: String,
    pub message: Option<MessageRow>,
    pub permission_id: Option<String>,
}

struct InFlightRun {
    run_id: String,
    task_id: String,
    conversation_id: String,
    cancel: Arc<AtomicBool>,
}

pub struct FlowStateCore {
    data_dir: PathBuf,
    db: Mutex<Database>,
    vault: CredentialVault,
    bus: EventBus,
    active_conversation: Mutex<Option<String>>,
    in_flight: Mutex<Option<InFlightRun>>,
    voice_listening: Mutex<bool>,
    local_provider: Option<Arc<dyn TextModelProvider>>,
}

impl FlowStateCore {
    pub fn open(data_dir: PathBuf) -> CoreResult<Self> {
        Self::open_with_local_provider(data_dir, None)
    }

    pub fn open_with_local_provider(
        data_dir: PathBuf,
        local_provider: Option<Arc<dyn TextModelProvider>>,
    ) -> CoreResult<Self> {
        let db_path = data_dir.join("flowstate.sqlite");
        let db = Database::open(&db_path)?;
        if db.get_provider_config(DEFAULT_PROVIDER_ID)?.is_none() {
            db.upsert_provider_config(DEFAULT_PROVIDER_ID, DEFAULT_OPENAI_MODEL)?;
        }
        db.upsert_provider_route("local", "apple-foundation", "system")?;
        db.upsert_provider_route("cloud", DEFAULT_PROVIDER_ID, DEFAULT_OPENAI_MODEL)?;
        db.upsert_provider_route("speechInput", "apple-speech", "en-IN")?;
        db.upsert_provider_route("speechOutput", "apple-speech", "system")?;
        let vault = CredentialVault::new(data_dir.join("credentials.json"))?;
        Ok(Self {
            data_dir,
            db: Mutex::new(db),
            vault,
            bus: EventBus::new(512),
            active_conversation: Mutex::new(None),
            in_flight: Mutex::new(None),
            voice_listening: Mutex::new(false),
            local_provider,
        })
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn health(&self) -> CoreHealth {
        CoreHealth {
            status: "ok".to_string(),
            version: CORE_VERSION.to_string(),
            protocol_version: SUPPORTED_SCHEMA_VERSION,
        }
    }

    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<Event> {
        self.bus.subscribe()
    }

    pub fn shutdown(&self) -> CoreResult<()> {
        let _ = self.cancel_active_run();
        tracing::info!("FlowState Core shutting down");
        Ok(())
    }

    pub fn set_active_conversation(&self, conversation_id: &str) -> CoreResult<()> {
        *self.active_conversation.lock().expect("lock") = Some(conversation_id.to_string());
        Ok(())
    }

    pub fn get_active_conversation(&self) -> Option<String> {
        self.active_conversation.lock().expect("lock").clone()
    }

    pub fn session_snapshot(&self) -> CoreResult<SessionSnapshot> {
        let conversation_id = self.get_active_conversation();
        let in_flight = self.in_flight.lock().expect("lock").clone();
        let voice_listening = *self.voice_listening.lock().expect("lock");

        let (active_task, active_run) = if let Some(in_flight) = in_flight {
            let db = self.db.lock().expect("db lock");
            (
                db.get_task(&in_flight.task_id)?,
                db.get_run(&in_flight.run_id)?,
            )
        } else {
            (None, None)
        };

        Ok(SessionSnapshot {
            active_conversation_id: conversation_id,
            active_task,
            active_run,
            voice_listening,
        })
    }

    pub fn list_events(&self, limit: usize) -> CoreResult<Vec<Event>> {
        let db = self.db.lock().expect("db lock");
        db.list_events(limit)
    }

    pub fn emit(&self, event: Event) -> CoreResult<Event> {
        validate_event(&event)?;
        {
            let db = self.db.lock().expect("db lock");
            db.insert_event(&event)?;
        }
        self.bus.publish(event.clone());
        Ok(event)
    }

    pub fn create_conversation(&self, title: &str) -> CoreResult<ConversationRow> {
        let db = self.db.lock().expect("db lock");
        let row = db.create_conversation(title)?;
        drop(db);
        self.set_active_conversation(&row.id)?;
        Ok(row)
    }

    pub fn list_conversations(&self) -> CoreResult<Vec<ConversationRow>> {
        let db = self.db.lock().expect("db lock");
        db.list_conversations()
    }

    pub fn list_messages(&self, conversation_id: &str) -> CoreResult<Vec<MessageRow>> {
        let db = self.db.lock().expect("db lock");
        db.list_messages(conversation_id)
    }

    pub fn list_tasks_for_conversation(
        &self,
        conversation_id: &str,
        limit: usize,
    ) -> CoreResult<Vec<TaskRow>> {
        let db = self.db.lock().expect("db lock");
        db.list_tasks_for_conversation(conversation_id, limit)
    }

    pub fn provider_status(&self, provider_id: &str) -> CoreResult<ProviderStatus> {
        let db = self.db.lock().expect("db lock");
        let config = db
            .get_provider_config(provider_id)?
            .unwrap_or(ProviderConfigRow {
                provider_id: provider_id.to_string(),
                default_model: DEFAULT_OPENAI_MODEL.to_string(),
                updated_at: Utc::now(),
            });
        let connected = self.vault.has_credential(provider_id)?;
        Ok(ProviderStatus {
            provider_id: provider_id.to_string(),
            connected,
            default_model: config.default_model,
        })
    }

    pub fn set_provider_api_key(
        &self,
        provider_id: &str,
        api_key: &str,
    ) -> CoreResult<ProviderStatus> {
        self.vault.set_api_key(provider_id, api_key)?;
        self.provider_status(provider_id)
    }

    pub fn remove_provider_api_key(&self, provider_id: &str) -> CoreResult<ProviderStatus> {
        self.vault.remove_api_key(provider_id)?;
        self.provider_status(provider_id)
    }

    pub fn set_provider_default_model(
        &self,
        provider_id: &str,
        model: &str,
    ) -> CoreResult<ProviderPublicConfig> {
        let db = self.db.lock().expect("db lock");
        let row = db.upsert_provider_config(provider_id, model)?;
        Ok(ProviderPublicConfig {
            provider_id: row.provider_id,
            default_model: row.default_model,
        })
    }

    pub fn voice_listening_started(&self, conversation_id: &str) -> CoreResult<Event> {
        *self.voice_listening.lock().expect("lock") = true;
        self.set_active_conversation(conversation_id)?;
        self.emit(new_event(
            "agent.started",
            None,
            json!({
                "conversationId": conversation_id,
                "phase": "listening",
                "source": "hud",
            }),
        ))
    }

    pub fn voice_listening_stopped(&self, conversation_id: &str) -> CoreResult<Event> {
        *self.voice_listening.lock().expect("lock") = false;
        self.emit(new_event(
            "agent.message",
            None,
            json!({
                "conversationId": conversation_id,
                "phase": "listening",
                "partial": false,
                "text": "",
                "kind": "listening.stopped",
            }),
        ))
    }

    pub fn voice_transcript_partial(&self, conversation_id: &str, text: &str) -> CoreResult<Event> {
        self.emit(new_event(
            "agent.message",
            None,
            json!({
                "conversationId": conversation_id,
                "partial": true,
                "text": text,
                "source": "hud",
                "kind": "transcript.partial",
                "providerId": "apple-speech",
                "locale": "en-IN",
                "execution": "local",
            }),
        ))
    }

    pub fn voice_transcript_final(&self, conversation_id: &str, text: &str) -> CoreResult<Event> {
        self.emit(new_event(
            "agent.message",
            None,
            json!({
                "conversationId": conversation_id,
                "partial": false,
                "text": text,
                "source": "hud",
                "kind": "transcript.final",
                "providerId": "apple-speech",
                "locale": "en-IN",
                "execution": "local",
            }),
        ))
    }

    pub fn cancel_active_run(&self) -> CoreResult<bool> {
        let in_flight = self.in_flight.lock().expect("lock").take();
        let Some(in_flight) = in_flight else {
            return Ok(false);
        };
        in_flight.cancel.store(true, Ordering::Relaxed);
        if let (Some(provider), Ok(handle)) =
            (&self.local_provider, tokio::runtime::Handle::try_current())
        {
            let provider = Arc::clone(provider);
            let conversation_id = in_flight.conversation_id.clone();
            handle.spawn(async move {
                let _ = provider.cancel(&conversation_id).await;
            });
        }
        {
            let db = self.db.lock().expect("db lock");
            db.update_run_status(&in_flight.run_id, "cancelled")?;
            db.update_task_status(&in_flight.task_id, "cancelled")?;
        }
        let _ = self.emit(new_event(
            "task.failed",
            Some(in_flight.run_id.clone()),
            json!({
                "taskId": in_flight.task_id,
                "conversationId": in_flight.conversation_id,
                "error": "cancelled",
                "cancelled": true,
            }),
        ));
        Ok(true)
    }

    pub fn trigger_dev_run(&self) -> CoreResult<Vec<Event>> {
        let db = self.db.lock().expect("db lock");
        let task: TaskRow = db.insert_task("Dev synthetic run", None)?;
        let run = db.insert_run(&task.id)?;
        drop(db);

        let mut emitted = Vec::new();
        emitted.push(self.emit(new_event(
            "task.started",
            Some(run.id.clone()),
            json!({ "taskId": task.id }),
        ))?);
        emitted.push(self.emit(new_event(
            "model.started",
            Some(run.id.clone()),
            json!({ "providerId": "dev", "model": "synthetic" }),
        ))?);
        emitted.push(self.emit(new_event(
            "model.delta",
            Some(run.id.clone()),
            json!({ "delta": "Synthetic stream chunk. " }),
        ))?);
        emitted.push(self.emit(new_event(
            "model.completed",
            Some(run.id.clone()),
            json!({ "providerId": "dev", "model": "synthetic" }),
        ))?);
        emitted.push(self.emit(new_event(
            "task.completed",
            Some(run.id),
            json!({ "taskId": task.id }),
        ))?);
        Ok(emitted)
    }

    pub async fn send_user_message(
        self: &Arc<Self>,
        conversation_id: &str,
        content: &str,
        source: &str,
    ) -> CoreResult<SendMessageOutcome> {
        self.set_active_conversation(conversation_id)?;
        let _ = self.cancel_active_run();

        {
            let db = self.db.lock().expect("db lock");
            db.insert_message(conversation_id, "user", content)?;
        }

        let task = {
            let db = self.db.lock().expect("db lock");
            db.insert_task("Ask", Some(conversation_id))?
        };

        let assessment = if let Some(provider) = &self.local_provider {
            provider.assess(content).await.unwrap_or_else(|error| {
                crate::providers::RouteAssessment {
                    route: ModelRoute::Cloud,
                    reason_code: format!(
                        "local_unavailable:{}",
                        compact_reason(&error.to_string())
                    ),
                }
            })
        } else {
            crate::providers::RouteAssessment {
                route: ModelRoute::Cloud,
                reason_code: "local_provider_unavailable".to_string(),
            }
        };

        if assessment.route == ModelRoute::Cloud {
            return self.request_cloud_permission(
                &task,
                conversation_id,
                content,
                source,
                &assessment.reason_code,
            );
        }

        self.execute_local(task, conversation_id, content, source)
            .await
    }

    pub async fn resolve_cloud_permission(
        self: &Arc<Self>,
        permission_id: &str,
        approved: bool,
    ) -> CoreResult<SendMessageOutcome> {
        let permission = {
            let db = self.db.lock().expect("db lock");
            db.get_pending_permission(permission_id)?
                .ok_or_else(|| CoreError::NotFound(format!("permission {permission_id}")))?
        };
        if permission.status != "pending" {
            return Err(CoreError::Provider(
                "permission_already_resolved".to_string(),
            ));
        }
        {
            let db = self.db.lock().expect("db lock");
            db.update_permission_status(
                permission_id,
                if approved { "approved" } else { "denied" },
            )?;
            if !approved {
                db.update_task_status(&permission.task_id, "cancelled")?;
            }
        }
        self.emit(new_event(
            "permission.resolved",
            None,
            json!({
                "permissionId": permission_id,
                "taskId": permission.task_id,
                "conversationId": permission.conversation_id,
                "approved": approved,
                "providerId": DEFAULT_PROVIDER_ID,
            }),
        ))?;
        if !approved {
            let message = {
                let db = self.db.lock().expect("db lock");
                db.insert_message(
                    &permission.conversation_id,
                    "assistant",
                    "This request is unavailable locally, and cloud access was not approved.",
                )?
            };
            self.emit(new_event(
                "agent.message",
                None,
                json!({
                    "conversationId": permission.conversation_id,
                    "partial": false,
                    "text": message.content,
                    "kind": "cloud.declined",
                }),
            ))?;
            self.emit(new_event(
                "task.failed",
                None,
                json!({
                    "taskId": permission.task_id,
                    "conversationId": permission.conversation_id,
                    "error": "cloud_declined",
                    "cancelled": true,
                }),
            ))?;
            return Ok(SendMessageOutcome {
                status: "declined".to_string(),
                message: Some(message),
                permission_id: Some(permission_id.to_string()),
            });
        }
        self.execute_cloud(permission).await
    }

    fn request_cloud_permission(
        &self,
        task: &TaskRow,
        conversation_id: &str,
        content: &str,
        source: &str,
        reason_code: &str,
    ) -> CoreResult<SendMessageOutcome> {
        let permission = {
            let db = self.db.lock().expect("db lock");
            db.insert_pending_permission(conversation_id, &task.id, content, source, reason_code)?
        };
        self.emit(new_event(
            "permission.requested",
            None,
            json!({
                "permissionId": permission.id,
                "taskId": task.id,
                "conversationId": conversation_id,
                "providerId": DEFAULT_PROVIDER_ID,
                "reasonCode": reason_code,
                "source": source,
                "summary": "This request is better suited to the connected cloud model. Send it off-device?",
            }),
        ))?;
        Ok(SendMessageOutcome {
            status: "needsApproval".to_string(),
            message: None,
            permission_id: Some(permission.id),
        })
    }

    async fn execute_local(
        self: &Arc<Self>,
        task: TaskRow,
        conversation_id: &str,
        content: &str,
        source: &str,
    ) -> CoreResult<SendMessageOutcome> {
        let provider = self.local_provider.as_ref().ok_or_else(|| {
            CoreError::Provider("Apple local provider is unavailable".to_string())
        })?;
        let history = {
            let db = self.db.lock().expect("db lock");
            db.list_messages(conversation_id)?
        };
        let request = ModelRequest {
            conversation_id: conversation_id.to_string(),
            prompt: content.to_string(),
            history: history
                .into_iter()
                .map(|message| ChatMessage {
                    role: message.role,
                    content: message.content,
                })
                .collect(),
        };
        let provider_id = provider.provider_id().to_string();
        let model = provider.model_name().to_string();
        let cancel = Arc::new(AtomicBool::new(false));
        let (run, assistant_id) = self.begin_run(
            &task,
            conversation_id,
            source,
            &provider_id,
            &model,
            &cancel,
        )?;
        let core = Arc::clone(self);
        let run_id = run.id.clone();
        let conversation = conversation_id.to_string();
        let source_owned = source.to_string();
        let assistant_for_delta = assistant_id.clone();
        let sink: DeltaSink = Arc::new(move |delta| {
            core.emit(new_event(
                "model.delta",
                Some(run_id.clone()),
                json!({
                    "delta": delta, "conversationId": conversation,
                    "messageId": assistant_for_delta, "source": source_owned,
                    "providerId": "apple-foundation", "execution": "local",
                }),
            ))?;
            Ok(())
        });
        let started_at = Instant::now();
        let result = provider.stream(request, Arc::clone(&cancel), sink).await;
        self.finish_run(
            result,
            task,
            run,
            conversation_id,
            source,
            &provider_id,
            &model,
            &assistant_id,
            "local",
            started_at.elapsed().as_millis() as u64,
        )
    }

    async fn execute_cloud(
        self: &Arc<Self>,
        permission: PendingPermissionRow,
    ) -> CoreResult<SendMessageOutcome> {
        let api_key = self
            .vault
            .get_api_key(DEFAULT_PROVIDER_ID)?
            .ok_or_else(|| CoreError::CredentialMissing(DEFAULT_PROVIDER_ID.to_string()))?;
        let model = {
            let db = self.db.lock().expect("db lock");
            db.get_provider_config(DEFAULT_PROVIDER_ID)?
                .map(|c| c.default_model)
                .unwrap_or_else(|| DEFAULT_OPENAI_MODEL.to_string())
        };
        let history = {
            let db = self.db.lock().expect("db lock");
            db.list_messages(&permission.conversation_id)?
        };
        let task = {
            let db = self.db.lock().expect("db lock");
            db.get_task(&permission.task_id)?
                .ok_or_else(|| CoreError::NotFound(permission.task_id.clone()))?
        };
        let cancel = Arc::new(AtomicBool::new(false));
        let (run, assistant_id) = self.begin_run(
            &task,
            &permission.conversation_id,
            &permission.source,
            DEFAULT_PROVIDER_ID,
            &model,
            &cancel,
        )?;
        let core = Arc::clone(self);
        let run_id = run.id.clone();
        let conversation = permission.conversation_id.clone();
        let source = permission.source.clone();
        let message_id = assistant_id.clone();
        let chat_messages = history
            .into_iter()
            .map(|message| ChatMessage {
                role: message.role,
                content: message.content,
            })
            .collect::<Vec<_>>();
        let started_at = Instant::now();
        let result = stream_openai_chat(&api_key, &model, &chat_messages, cancel, move |delta| {
            core.emit(new_event(
                "model.delta",
                Some(run_id.clone()),
                json!({
                    "delta": delta, "conversationId": conversation, "messageId": message_id,
                    "source": source, "providerId": DEFAULT_PROVIDER_ID, "execution": "cloud",
                }),
            ))?;
            Ok(())
        })
        .await;
        self.finish_run(
            result,
            task,
            run,
            &permission.conversation_id,
            &permission.source,
            DEFAULT_PROVIDER_ID,
            &model,
            &assistant_id,
            "cloud",
            started_at.elapsed().as_millis() as u64,
        )
    }

    fn begin_run(
        &self,
        task: &TaskRow,
        conversation_id: &str,
        source: &str,
        provider_id: &str,
        model: &str,
        cancel: &Arc<AtomicBool>,
    ) -> CoreResult<(RunRow, String)> {
        let run = {
            let db = self.db.lock().expect("db lock");
            db.update_task_status(&task.id, "running")?;
            db.insert_run(&task.id)?
        };
        {
            *self.in_flight.lock().expect("lock") = Some(InFlightRun {
                run_id: run.id.clone(),
                task_id: task.id.clone(),
                conversation_id: conversation_id.to_string(),
                cancel: Arc::clone(cancel),
            });
        }
        self.emit(new_event(
            "task.started",
            Some(run.id.clone()),
            json!({ "taskId": task.id, "conversationId": conversation_id, "source": source }),
        ))?;
        self.emit(new_event(
            "model.started",
            Some(run.id.clone()),
            json!({
                "providerId": provider_id,
                "model": model,
                "conversationId": conversation_id,
                "source": source,
                "execution": if provider_id == DEFAULT_PROVIDER_ID { "cloud" } else { "local" },
            }),
        ))?;
        Ok((run, Uuid::new_v4().to_string()))
    }

    fn finish_run(
        &self,
        stream_result: CoreResult<String>,
        task: TaskRow,
        run: RunRow,
        conversation_id: &str,
        source: &str,
        provider_id: &str,
        model: &str,
        assistant_id: &str,
        execution: &str,
        duration_ms: u64,
    ) -> CoreResult<SendMessageOutcome> {
        *self.in_flight.lock().expect("lock") = None;
        match stream_result {
            Ok(full_text) => {
                {
                    let db = self.db.lock().expect("db lock");
                    db.insert_message_with_id(
                        assistant_id,
                        conversation_id,
                        "assistant",
                        &full_text,
                    )?;
                    db.update_run_status(&run.id, "completed")?;
                    db.update_task_status(&task.id, "completed")?;
                }
                self.emit(new_event(
                    "model.completed",
                    Some(run.id.clone()),
                    json!({
                        "providerId": provider_id,
                        "model": model,
                        "conversationId": conversation_id,
                        "messageId": assistant_id,
                        "source": source,
                        "text": full_text,
                        "execution": execution,
                        "timing": { "durationMs": duration_ms },
                    }),
                ))?;
                self.emit(new_event(
                    "task.completed",
                    Some(run.id),
                    json!({ "taskId": task.id, "conversationId": conversation_id }),
                ))?;
                Ok(SendMessageOutcome {
                    status: "completed".to_string(),
                    message: Some(MessageRow {
                        id: assistant_id.to_string(),
                        conversation_id: conversation_id.to_string(),
                        role: "assistant".to_string(),
                        content: full_text,
                        created_at: Utc::now(),
                    }),
                    permission_id: None,
                })
            }
            Err(CoreError::Cancelled) => Err(CoreError::Cancelled),
            Err(err) => {
                let message = err.to_string();
                {
                    let db = self.db.lock().expect("db lock");
                    db.update_run_status(&run.id, "failed")?;
                    db.update_task_status(&task.id, "failed")?;
                }
                let _ = self.emit(new_event(
                    "task.failed",
                    Some(run.id),
                    json!({ "taskId": task.id, "error": message, "conversationId": conversation_id }),
                ));
                Err(err)
            }
        }
    }
}

fn compact_reason(reason: &str) -> String {
    reason
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || *character == '_')
        .take(48)
        .collect::<String>()
        .to_lowercase()
}

impl Clone for InFlightRun {
    fn clone(&self) -> Self {
        Self {
            run_id: self.run_id.clone(),
            task_id: self.task_id.clone(),
            conversation_id: self.conversation_id.clone(),
            cancel: Arc::clone(&self.cancel),
        }
    }
}

fn new_event(type_name: &str, run_id: Option<String>, payload: Value) -> Event {
    let value = json!({
        "id": Uuid::new_v4().to_string(),
        "schemaVersion": SUPPORTED_SCHEMA_VERSION,
        "type": type_name,
        "occurredAt": Utc::now().to_rfc3339(),
        "runId": run_id,
        "payload": payload,
    });
    serde_json::from_value(value).expect("valid protocol event")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::validate_event;
    use crate::providers::{RouteAssessment, TextModelProvider};
    use async_trait::async_trait;

    struct MockLocalProvider {
        route: ModelRoute,
    }

    #[async_trait]
    impl TextModelProvider for MockLocalProvider {
        fn provider_id(&self) -> &'static str {
            "mock-local"
        }
        fn model_name(&self) -> &'static str {
            "mock"
        }
        async fn assess(&self, _prompt: &str) -> CoreResult<RouteAssessment> {
            Ok(RouteAssessment {
                route: self.route.clone(),
                reason_code: "test_route".to_string(),
            })
        }
        async fn stream(
            &self,
            _request: ModelRequest,
            _cancel: Arc<AtomicBool>,
            on_delta: DeltaSink,
        ) -> CoreResult<String> {
            on_delta("Hello ".to_string())?;
            on_delta("locally.".to_string())?;
            Ok("Hello locally.".to_string())
        }
    }

    #[test]
    fn rejects_unsupported_schema_version() {
        let mut event = new_event("task.started", None, json!({}));
        event.schema_version = 2;
        assert!(validate_event(&event).is_err());
    }

    #[test]
    fn dev_run_emits_protocol_events() {
        let dir = tempfile::tempdir().expect("tempdir");
        let core = FlowStateCore::open(dir.path().to_path_buf()).expect("open");
        let events = core.trigger_dev_run().expect("dev run");
        assert_eq!(events.len(), 5);
        assert!(events.iter().all(|e| validate_event(e).is_ok()));
    }

    #[tokio::test]
    async fn local_route_streams_without_cloud_credentials() {
        let dir = tempfile::tempdir().expect("tempdir");
        let provider: Arc<dyn TextModelProvider> = Arc::new(MockLocalProvider {
            route: ModelRoute::Local,
        });
        let core = Arc::new(
            FlowStateCore::open_with_local_provider(dir.path().to_path_buf(), Some(provider))
                .expect("open"),
        );
        let conversation = core.create_conversation("test").expect("conversation");
        let outcome = core
            .send_user_message(&conversation.id, "Rewrite this", "text")
            .await
            .expect("local response");
        assert_eq!(outcome.status, "completed");
        assert_eq!(outcome.message.expect("message").content, "Hello locally.");
        assert!(core
            .list_events(50)
            .expect("events")
            .iter()
            .any(|event| { event.type_.to_string() == "model.delta" }));
    }

    #[tokio::test]
    async fn cloud_route_stops_at_persisted_permission_gate() {
        let dir = tempfile::tempdir().expect("tempdir");
        let provider: Arc<dyn TextModelProvider> = Arc::new(MockLocalProvider {
            route: ModelRoute::Cloud,
        });
        let core = Arc::new(
            FlowStateCore::open_with_local_provider(dir.path().to_path_buf(), Some(provider))
                .expect("open"),
        );
        let conversation = core.create_conversation("test").expect("conversation");
        let outcome = core
            .send_user_message(&conversation.id, "What happened today?", "voice")
            .await
            .expect("permission request");
        assert_eq!(outcome.status, "needsApproval");
        let permission_id = outcome.permission_id.expect("permission id");
        let events = core.list_events(50).expect("events");
        assert!(events
            .iter()
            .any(|event| event.type_.to_string() == "permission.requested"));
        assert!(!events
            .iter()
            .any(|event| event.type_.to_string() == "model.started"));
        let declined = core
            .resolve_cloud_permission(&permission_id, false)
            .await
            .expect("decline");
        assert_eq!(declined.status, "declined");
    }
}
