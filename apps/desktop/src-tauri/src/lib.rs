mod apple_provider;
mod apple_runtime;
mod hud;
mod system_permissions;

use std::sync::Arc;

use apple_provider::{AppleFoundationProvider, AppleSpeechOutputProvider, AppleSpeechProvider};
use apple_runtime::{AppleRuntime, AppleRuntimeStatus};

use flowstate_core::{
    ConversationRow, CoreHealth, FlowStateCore, MessageRow, ProviderPublicConfig, ProviderStatus,
    SendMessageOutcome, SessionSnapshot, SpeechInputProvider, SpeechOutputProvider, TaskRow,
    TextModelProvider, DEFAULT_PROVIDER_ID,
};
use flowstate_protocol::Event;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, RunEvent, State};

struct AppCore(pub Arc<FlowStateCore>);
struct AppleRuntimeState(pub Arc<AppleRuntime>);

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

fn apple_runtime_error(error: impl ToString) -> CommandError {
    CommandError {
        code: "apple_runtime_error".to_string(),
        message: error.to_string(),
    }
}

#[tauri::command]
async fn cancel_speech(runtime: State<'_, AppleRuntimeState>) -> Result<(), CommandError> {
    runtime.0.cancel_speech().await.map_err(apple_runtime_error)
}

#[tauri::command]
async fn speak_text(
    runtime: State<'_, AppleRuntimeState>,
    text: String,
) -> Result<(), CommandError> {
    runtime.0.speak(&text).await.map_err(apple_runtime_error)
}

#[tauri::command]
async fn apple_runtime_status(
    runtime: State<'_, AppleRuntimeState>,
) -> Result<AppleRuntimeStatus, CommandError> {
    runtime.0.status().await.map_err(apple_runtime_error)
}

#[tauri::command]
async fn install_speech_assets(
    runtime: State<'_, AppleRuntimeState>,
) -> Result<serde_json::Value, CommandError> {
    runtime
        .0
        .install_speech_assets()
        .await
        .map_err(apple_runtime_error)
}

#[tauri::command]
fn get_system_permissions() -> Vec<system_permissions::SystemPermission> {
    system_permissions::list()
}

#[tauri::command]
async fn request_system_permission(
    permission_id: String,
) -> Result<Vec<system_permissions::SystemPermission>, CommandError> {
    match permission_id.as_str() {
        "microphone" => {
            let status = system_permissions::list()
                .into_iter()
                .find(|permission| permission.id() == "microphone")
                .map(|permission| permission.status().to_string())
                .unwrap_or_else(|| "restricted".to_string());
            if status == "not_determined" {
                system_permissions::request_microphone().await?;
            } else if status != "granted" {
                system_permissions::open_settings("microphone")?;
            }
        }
        "accessibility" => system_permissions::open_settings("accessibility")?,
        "screen" => system_permissions::request_screen_recording(),
        _ => {
            return Err(CommandError {
                code: "invalid_permission".to_string(),
                message: format!("Unknown system permission: {permission_id}"),
            })
        }
    }
    Ok(system_permissions::list())
}

#[tauri::command]
fn open_system_permission_settings(permission_id: String) -> Result<(), CommandError> {
    system_permissions::open_settings(&permission_id)
}

#[tauri::command]
async fn start_voice_capture(
    core: State<'_, AppCore>,
    runtime: State<'_, AppleRuntimeState>,
    conversation_id: String,
) -> Result<String, CommandError> {
    let _ = core.0.cancel_active_run();
    core.0.voice_listening_started(&conversation_id)?;
    match runtime.0.start_capture(conversation_id).await {
        Ok(id) => Ok(id),
        Err(error) => {
            let _ = core.0.voice_listening_stopped(
                core.0
                    .get_active_conversation()
                    .as_deref()
                    .unwrap_or_default(),
            );
            Err(apple_runtime_error(error))
        }
    }
}

#[tauri::command]
async fn stop_voice_capture(
    core: State<'_, AppCore>,
    runtime: State<'_, AppleRuntimeState>,
) -> Result<Option<SendMessageOutcome>, CommandError> {
    let conversation_id = core
        .0
        .get_active_conversation()
        .ok_or_else(|| CommandError {
            code: "not_found".to_string(),
            message: "No active conversation.".to_string(),
        })?;
    let transcript = runtime
        .0
        .stop_capture()
        .await
        .map_err(apple_runtime_error)?;
    core.0.voice_listening_stopped(&conversation_id)?;
    if transcript.is_empty() {
        return Ok(None);
    }
    core.0
        .voice_transcript_final(&conversation_id, &transcript)?;
    core.0
        .send_user_message(&conversation_id, &transcript, "voice")
        .await
        .map(Some)
        .map_err(Into::into)
}

#[tauri::command]
async fn cancel_voice_capture(
    core: State<'_, AppCore>,
    runtime: State<'_, AppleRuntimeState>,
) -> Result<(), CommandError> {
    runtime
        .0
        .cancel_capture()
        .await
        .map_err(apple_runtime_error)?;
    if let Some(conversation_id) = core.0.get_active_conversation() {
        core.0.voice_listening_stopped(&conversation_id)?;
    }
    Ok(())
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
fn set_active_conversation(
    core: State<'_, AppCore>,
    conversation_id: String,
) -> Result<(), CommandError> {
    core.0
        .set_active_conversation(&conversation_id)
        .map_err(Into::into)
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
fn create_conversation(
    core: State<'_, AppCore>,
    title: String,
) -> Result<ConversationRow, CommandError> {
    core.0.create_conversation(&title).map_err(Into::into)
}

#[tauri::command]
fn list_conversations(core: State<'_, AppCore>) -> Result<Vec<ConversationRow>, CommandError> {
    core.0.list_conversations().map_err(Into::into)
}

#[tauri::command]
fn list_messages(
    core: State<'_, AppCore>,
    conversation_id: String,
) -> Result<Vec<MessageRow>, CommandError> {
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
fn provider_status(
    core: State<'_, AppCore>,
    provider_id: Option<String>,
) -> Result<ProviderStatus, CommandError> {
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
        .set_provider_api_key(
            provider_id.as_deref().unwrap_or(DEFAULT_PROVIDER_ID),
            &api_key,
        )
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
        .set_provider_default_model(
            provider_id.as_deref().unwrap_or(DEFAULT_PROVIDER_ID),
            &model,
        )
        .map_err(Into::into)
}

#[tauri::command]
async fn send_message(
    core: State<'_, AppCore>,
    conversation_id: String,
    content: String,
    source: Option<String>,
) -> Result<SendMessageOutcome, CommandError> {
    core.0
        .send_user_message(
            &conversation_id,
            &content,
            source.as_deref().unwrap_or("text"),
        )
        .await
        .map_err(Into::into)
}

#[tauri::command]
async fn resolve_cloud_approval(
    core: State<'_, AppCore>,
    permission_id: String,
    approved: bool,
) -> Result<SendMessageOutcome, CommandError> {
    core.0
        .resolve_cloud_permission(&permission_id, approved)
        .await
        .map_err(Into::into)
}

#[tauri::command]
fn cancel_active_run(core: State<'_, AppCore>) -> Result<bool, CommandError> {
    core.0.cancel_active_run().map_err(Into::into)
}

#[tauri::command]
fn voice_listening_started(
    core: State<'_, AppCore>,
    conversation_id: String,
) -> Result<Event, CommandError> {
    core.0
        .voice_listening_started(&conversation_id)
        .map_err(Into::into)
}

#[tauri::command]
fn voice_listening_stopped(
    core: State<'_, AppCore>,
    conversation_id: String,
) -> Result<Event, CommandError> {
    core.0
        .voice_listening_stopped(&conversation_id)
        .map_err(Into::into)
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
            core.0
                .set_active_conversation(&id)
                .map_err(|e| CommandError {
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

fn spawn_apple_event_forwarder(
    app: &AppHandle,
    core: Arc<FlowStateCore>,
    runtime: Arc<AppleRuntime>,
) {
    let handle = app.clone();
    let mut events = runtime.subscribe();
    tauri::async_runtime::spawn(async move {
        loop {
            match events.recv().await {
                Ok(event) => {
                    let _ = handle.emit("apple-runtime-event", &event);
                    if event.event != "transcript" {
                        continue;
                    }
                    let Some(context) = runtime.capture_context() else {
                        continue;
                    };
                    if context.request_id != event.id {
                        continue;
                    }
                    let is_final = event
                        .payload
                        .get("isFinal")
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(false);
                    if is_final {
                        continue;
                    }
                    if let Some(text) = event
                        .payload
                        .get("text")
                        .and_then(serde_json::Value::as_str)
                    {
                        let _ = core.voice_transcript_partial(&context.conversation_id, text);
                    }
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
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("app data dir")
                .join("core");
            std::fs::create_dir_all(&data_dir).expect("create core data dir");
            let apple_runtime = AppleRuntime::start(app.handle().clone());
            let local_provider: Arc<dyn TextModelProvider> =
                Arc::new(AppleFoundationProvider::new(Arc::clone(&apple_runtime)));
            let speech_input = AppleSpeechProvider;
            let speech_output = AppleSpeechOutputProvider;
            let _provider_roles = (speech_input.provider_id(), speech_output.provider_id());
            let core = Arc::new(
                FlowStateCore::open_with_local_provider(data_dir, Some(local_provider))
                    .expect("open FlowState Core"),
            );
            spawn_event_forwarder(app.handle(), Arc::clone(&core));
            spawn_apple_event_forwarder(
                app.handle(),
                Arc::clone(&core),
                Arc::clone(&apple_runtime),
            );
            app.manage(AppCore(core));
            app.manage(AppleRuntimeState(apple_runtime));
            hud::start(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            hud::show_hud,
            hud::hide_hud,
            hud::hud_ready,
            hud::set_hud_busy,
            hud::set_hud_surface,
            cancel_speech,
            speak_text,
            apple_runtime_status,
            install_speech_assets,
            get_system_permissions,
            request_system_permission,
            open_system_permission_settings,
            start_voice_capture,
            stop_voice_capture,
            cancel_voice_capture,
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
            resolve_cloud_approval,
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
                if let Some(hud) = app_handle.try_state::<hud::HudState>() {
                    hud.stopped
                        .store(true, std::sync::atomic::Ordering::Relaxed);
                }
                if let Some(runtime) = app_handle.try_state::<AppleRuntimeState>() {
                    runtime.0.shutdown();
                }
                if let Some(core) = app_handle.try_state::<AppCore>() {
                    let _ = core.0.shutdown();
                }
            }
        });
}
