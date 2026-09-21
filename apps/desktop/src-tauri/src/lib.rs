use std::sync::Arc;

use flowstate_core::{
    ConversationRow, CoreHealth, FlowStateCore, MessageRow, ProviderPublicConfig, ProviderStatus,
    SessionSnapshot, TaskRow, DEFAULT_PROVIDER_ID,
};
use flowstate_protocol::Event;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, RunEvent, State};

struct AppCore(pub Arc<FlowStateCore>);

#[derive(Serialize)]
struct CommandError {
    code: String,
    message: String,
}

impl From<flowstate_core::CoreError> for CommandError {
    fn from(value: flowstate_core::CoreError) -> Self {
        let code = match &value {
            flowstate_core::CoreError::CredentialMissing(_) => "credential_missing",
            flowstate_core::CoreError::UnsupportedSchemaVersion(_) => "unsupported_schema_version",
            flowstate_core::CoreError::Provider(_) => "provider_error",
            flowstate_core::CoreError::NotFound(_) => "not_found",
            flowstate_core::CoreError::Cancelled => "cancelled",
            _ => "core_error",
        };
        Self {
            code: code.to_string(),
            message: value.to_string(),
        }
    }
}

fn toggle_hud_window(app: &AppHandle) {
    let Some(hud) = app.get_webview_window("hud") else {
        return;
    };
    if hud.is_visible().unwrap_or(false) {
        let _ = hud.hide();
    } else {
        let _ = hud.center();
        let _ = hud.show();
        let _ = hud.set_focus();
    }
}

#[tauri::command]
fn toggle_hud(app: AppHandle) {
    toggle_hud_window(&app);
}

#[tauri::command]
fn core_health(core: State<'_, AppCore>) -> CoreHealth {
    core.0.health()
}

#[tauri::command]
fn session_snapshot(core: State<'_, AppCore>) -> Result<SessionSnapshot, CommandError> {
    core.0.session_snapshot().map_err(Into::into)
}

#[tauri::command]
fn set_active_conversation(core: State<'_, AppCore>, conversation_id: String) -> Result<(), CommandError> {
    core.0.set_active_conversation(&conversation_id).map_err(Into::into)
}

#[tauri::command]
fn list_events(core: State<'_, AppCore>, limit: Option<usize>) -> Result<Vec<Event>, CommandError> {
    core.0.list_events(limit.unwrap_or(200)).map_err(Into::into)
}

#[tauri::command]
fn trigger_dev_run(core: State<'_, AppCore>) -> Result<Vec<Event>, CommandError> {
    core.0.trigger_dev_run().map_err(Into::into)
}

#[tauri::command]
fn create_conversation(core: State<'_, AppCore>, title: String) -> Result<ConversationRow, CommandError> {
    core.0.create_conversation(&title).map_err(Into::into)
}

#[tauri::command]
fn list_conversations(core: State<'_, AppCore>) -> Result<Vec<ConversationRow>, CommandError> {
    core.0.list_conversations().map_err(Into::into)
}

#[tauri::command]
fn list_messages(core: State<'_, AppCore>, conversation_id: String) -> Result<Vec<MessageRow>, CommandError> {
    core.0.list_messages(&conversation_id).map_err(Into::into)
}

#[tauri::command]
fn list_tasks_for_conversation(
    core: State<'_, AppCore>,
    conversation_id: String,
    limit: Option<usize>,
) -> Result<Vec<TaskRow>, CommandError> {
    core.0
        .list_tasks_for_conversation(&conversation_id, limit.unwrap_or(20))
        .map_err(Into::into)
}

#[tauri::command]
fn provider_status(core: State<'_, AppCore>, provider_id: Option<String>) -> Result<ProviderStatus, CommandError> {
    core.0
        .provider_status(provider_id.as_deref().unwrap_or(DEFAULT_PROVIDER_ID))
        .map_err(Into::into)
}

#[tauri::command]
fn set_provider_api_key(
    core: State<'_, AppCore>,
    provider_id: Option<String>,
    api_key: String,
) -> Result<ProviderStatus, CommandError> {
    core.0
        .set_provider_api_key(provider_id.as_deref().unwrap_or(DEFAULT_PROVIDER_ID), &api_key)
        .map_err(Into::into)
}

#[tauri::command]
fn remove_provider_api_key(
    core: State<'_, AppCore>,
    provider_id: Option<String>,
) -> Result<ProviderStatus, CommandError> {
    core.0
        .remove_provider_api_key(provider_id.as_deref().unwrap_or(DEFAULT_PROVIDER_ID))
        .map_err(Into::into)
}

#[tauri::command]
fn set_provider_default_model(
    core: State<'_, AppCore>,
    provider_id: Option<String>,
    model: String,
) -> Result<ProviderPublicConfig, CommandError> {
    core.0
        .set_provider_default_model(provider_id.as_deref().unwrap_or(DEFAULT_PROVIDER_ID), &model)
        .map_err(Into::into)
}

#[tauri::command]
async fn send_message(
    core: State<'_, AppCore>,
    conversation_id: String,
    content: String,
    source: Option<String>,
) -> Result<MessageRow, CommandError> {
    core.0
        .send_user_message(&conversation_id, &content, source.as_deref().unwrap_or("text"))
        .await
        .map_err(Into::into)
}

#[tauri::command]
fn cancel_active_run(core: State<'_, AppCore>) -> Result<bool, CommandError> {
    core.0.cancel_active_run().map_err(Into::into)
}

#[tauri::command]
fn voice_listening_started(core: State<'_, AppCore>, conversation_id: String) -> Result<Event, CommandError> {
    core.0.voice_listening_started(&conversation_id).map_err(Into::into)
}

#[tauri::command]
fn voice_listening_stopped(core: State<'_, AppCore>, conversation_id: String) -> Result<Event, CommandError> {
    core.0.voice_listening_stopped(&conversation_id).map_err(Into::into)
}

#[tauri::command]
fn voice_transcript_partial(
    core: State<'_, AppCore>,
    conversation_id: String,
    text: String,
) -> Result<Event, CommandError> {
    core.0
        .voice_transcript_partial(&conversation_id, &text)
        .map_err(Into::into)
}

#[tauri::command]
fn focus_main_window(app: AppHandle, conversation_id: Option<String>) -> Result<(), CommandError> {
    if let Some(id) = conversation_id {
        if let Some(core) = app.try_state::<AppCore>() {
            core.0.set_active_conversation(&id).map_err(|e| CommandError {
                code: "core_error".to_string(),
                message: e.to_string(),
            })?;
        }
    }
    if let Some(main) = app.get_webview_window("main") {
        main.show().map_err(|e| CommandError {
            code: "window_error".to_string(),
            message: e.to_string(),
        })?;
        main.set_focus().map_err(|e| CommandError {
            code: "window_error".to_string(),
            message: e.to_string(),
        })?;
    }
    Ok(())
}

fn spawn_event_forwarder(app: &AppHandle, core: Arc<FlowStateCore>) {
    let handle = app.clone();
    let mut rx = core.subscribe();
    tauri::async_runtime::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    let _ = handle.emit("flowstate-event", event);
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(_) => break,
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("app data dir")
                .join("core");
            std::fs::create_dir_all(&data_dir).expect("create core data dir");
            let core = Arc::new(FlowStateCore::open(data_dir).expect("open FlowState Core"));
            spawn_event_forwarder(app.handle(), Arc::clone(&core));
            app.manage(AppCore(core));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            toggle_hud,
            core_health,
            session_snapshot,
            set_active_conversation,
            list_events,
            trigger_dev_run,
            create_conversation,
            list_conversations,
            list_messages,
            list_tasks_for_conversation,
            provider_status,
            set_provider_api_key,
            remove_provider_api_key,
            set_provider_default_model,
            send_message,
            cancel_active_run,
            voice_listening_started,
            voice_listening_stopped,
            voice_transcript_partial,
            focus_main_window,
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app_handle, event| {
            if let RunEvent::Exit = event {
                if let Some(core) = app_handle.try_state::<AppCore>() {
                    let _ = core.0.shutdown();
                }
            }
        });
}
