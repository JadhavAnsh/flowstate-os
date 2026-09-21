use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use chrono::Utc;
use flowstate_protocol::Event;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::bus::EventBus;
use crate::db::{ConversationRow, Database, MessageRow, ProviderConfigRow, RunRow, TaskRow};
use crate::error::{CoreError, CoreResult};
use crate::events::{validate_event, SUPPORTED_SCHEMA_VERSION};
use crate::model::{stream_openai_chat, ChatMessage};
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
}

impl FlowStateCore {
    pub fn open(data_dir: PathBuf) -> CoreResult<Self> {
        let db_path = data_dir.join("flowstate.sqlite");
        let db = Database::open(&db_path)?;
        if db.get_provider_config(DEFAULT_PROVIDER_ID)?.is_none() {
            db.upsert_provider_config(DEFAULT_PROVIDER_ID, DEFAULT_OPENAI_MODEL)?;
        }
        let vault = CredentialVault::new(data_dir.join("credentials.json"))?;
        Ok(Self {
            data_dir,
            db: Mutex::new(db),
            vault,
            bus: EventBus::new(512),
            active_conversation: Mutex::new(None),
            in_flight: Mutex::new(None),
            voice_listening: Mutex::new(false),
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

    pub fn list_tasks_for_conversation(&self, conversation_id: &str, limit: usize) -> CoreResult<Vec<TaskRow>> {
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

    pub fn set_provider_api_key(&self, provider_id: &str, api_key: &str) -> CoreResult<ProviderStatus> {
        self.vault.set_api_key(provider_id, api_key)?;
        self.provider_status(provider_id)
    }

    pub fn remove_provider_api_key(&self, provider_id: &str) -> CoreResult<ProviderStatus> {
        self.vault.remove_api_key(provider_id)?;
        self.provider_status(provider_id)
    }

    pub fn set_provider_default_model(&self, provider_id: &str, model: &str) -> CoreResult<ProviderPublicConfig> {
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
            }),
        ))
    }

    pub fn cancel_active_run(&self) -> CoreResult<bool> {
        let in_flight = self.in_flight.lock().expect("lock").take();
        let Some(in_flight) = in_flight else {
            return Ok(false);
        };
        in_flight.cancel.store(true, Ordering::Relaxed);
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
    ) -> CoreResult<MessageRow> {
        self.set_active_conversation(conversation_id)?;
        let _ = self.cancel_active_run();

        {
            let db = self.db.lock().expect("db lock");
            db.insert_message(conversation_id, "user", content)?;
        }

        let provider_id = DEFAULT_PROVIDER_ID;
        let api_key = self.vault.get_api_key(provider_id)?
            .ok_or_else(|| CoreError::CredentialMissing(provider_id.to_string()))?;

        let model = {
            let db = self.db.lock().expect("db lock");
            db.get_provider_config(provider_id)?
                .map(|c| c.default_model)
                .unwrap_or_else(|| DEFAULT_OPENAI_MODEL.to_string())
        };

        let history = {
            let db = self.db.lock().expect("db lock");
            db.list_messages(conversation_id)?
        };

        let task = {
            let db = self.db.lock().expect("db lock");
            db.insert_task("Ask", Some(conversation_id))?
        };
        let run = {
            let db = self.db.lock().expect("db lock");
            db.update_task_status(&task.id, "running")?;
            db.insert_run(&task.id)?
        };

        let cancel = Arc::new(AtomicBool::new(false));
        {
            *self.in_flight.lock().expect("lock") = Some(InFlightRun {
                run_id: run.id.clone(),
                task_id: task.id.clone(),
                conversation_id: conversation_id.to_string(),
                cancel: Arc::clone(&cancel),
            });
        }

        let core = Arc::clone(self);
        let run_id = run.id.clone();
        let conversation_id_owned = conversation_id.to_string();

        self.emit(new_event(
            "task.started",
            Some(run_id.clone()),
            json!({
                "taskId": task.id,
                "conversationId": conversation_id,
                "source": source,
            }),
        ))?;
        self.emit(new_event(
            "model.started",
            Some(run_id.clone()),
            json!({
                "providerId": provider_id,
                "model": model,
                "conversationId": conversation_id,
                "source": source,
            }),
        ))?;

        let chat_messages: Vec<ChatMessage> = history
            .iter()
            .map(|m| ChatMessage {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect();

        let assistant_id = Uuid::new_v4().to_string();
        let stream_result = stream_openai_chat(
            &api_key,
            &model,
            &chat_messages,
            cancel,
            |delta| {
                core.emit(new_event(
                    "model.delta",
                    Some(run_id.clone()),
                    json!({
                        "delta": delta,
                        "conversationId": conversation_id_owned,
                        "messageId": assistant_id,
                        "source": source,
                    }),
                ))?;
                Ok(())
            },
        )
        .await;

        *core.in_flight.lock().expect("lock") = None;

        match stream_result {
            Ok(full_text) => {
                {
                    let db = core.db.lock().expect("db lock");
                    db.insert_message(&conversation_id_owned, "assistant", &full_text)?;
                    db.update_run_status(&run_id, "completed")?;
                    db.update_task_status(&task.id, "completed")?;
                }
                core.emit(new_event(
                    "model.completed",
                    Some(run_id.clone()),
                    json!({
                        "providerId": provider_id,
                        "model": model,
                        "conversationId": conversation_id_owned,
                        "messageId": assistant_id,
                        "source": source,
                        "text": full_text,
                    }),
                ))?;
                core.emit(new_event(
                    "task.completed",
                    Some(run_id),
                    json!({ "taskId": task.id, "conversationId": conversation_id_owned }),
                ))?;
                Ok(MessageRow {
                    id: assistant_id,
                    conversation_id: conversation_id_owned,
                    role: "assistant".to_string(),
                    content: full_text,
                    created_at: Utc::now(),
                })
            }
            Err(CoreError::Cancelled) => {
                Err(CoreError::Cancelled)
            }
            Err(err) => {
                let message = err.to_string();
                {
                    let db = core.db.lock().expect("db lock");
                    db.update_run_status(&run_id, "failed")?;
                    db.update_task_status(&task.id, "failed")?;
                }
                let _ = core.emit(new_event(
                    "task.failed",
                    Some(run_id.clone()),
                    json!({ "taskId": task.id, "error": message, "conversationId": conversation_id_owned }),
                ));
                Err(err)
            }
        }
    }
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
}
