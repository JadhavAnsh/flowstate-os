use serde::Serialize;

use crate::CommandError;

#[cfg(target_os = "macos")]
use objc2_av_foundation::{AVAuthorizationStatus, AVCaptureDevice, AVMediaTypeAudio};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemPermission {
    id: &'static str,
    status: String,
}

impl SystemPermission {
    pub fn id(&self) -> &str {
        self.id
    }

    pub fn status(&self) -> &str {
        &self.status
    }
}

#[cfg(target_os = "macos")]
fn accessibility_granted() -> bool {
    #[link(name = "ApplicationServices", kind = "framework")]
    unsafe extern "C" {
        fn AXIsProcessTrusted() -> u8;
    }

    // SAFETY: AXIsProcessTrusted takes no arguments and returns a CoreFoundation Boolean.
    unsafe { AXIsProcessTrusted() != 0 }
}

#[cfg(not(target_os = "macos"))]
fn accessibility_granted() -> bool {
    false
}

#[cfg(target_os = "macos")]
fn screen_recording_granted() -> bool {
    core_graphics::access::ScreenCaptureAccess.preflight()
}

#[cfg(not(target_os = "macos"))]
fn screen_recording_granted() -> bool {
    false
}

#[cfg(target_os = "macos")]
fn microphone_status() -> String {
    // SAFETY: AVMediaTypeAudio is an immutable AVFoundation framework constant.
    let Some(media_type) = (unsafe { AVMediaTypeAudio }) else {
        return "restricted".to_string();
    };
    // SAFETY: AVMediaTypeAudio is the exact media type required by this AVFoundation API.
    let status = unsafe { AVCaptureDevice::authorizationStatusForMediaType(media_type) };
    match status {
        AVAuthorizationStatus::Authorized => "granted",
        AVAuthorizationStatus::Denied => "denied",
        AVAuthorizationStatus::Restricted => "restricted",
        _ => "not_determined",
    }
    .to_string()
}

#[cfg(not(target_os = "macos"))]
fn microphone_status() -> String {
    "restricted".to_string()
}

#[cfg(target_os = "macos")]
pub async fn request_microphone() -> Result<(), CommandError> {
    use std::sync::mpsc;
    use std::time::Duration;

    use block2::RcBlock;
    use objc2::runtime::Bool;

    tokio::task::spawn_blocking(move || {
        // SAFETY: AVMediaTypeAudio is an immutable AVFoundation framework constant.
        let media_type = (unsafe { AVMediaTypeAudio }).ok_or_else(|| CommandError {
            code: "microphone_unavailable".to_string(),
            message: "The macOS microphone media type is unavailable.".to_string(),
        })?;
        let (sender, receiver) = mpsc::channel();
        let completion = RcBlock::new(move |_granted: Bool| {
            let _ = sender.send(());
        });
        // SAFETY: AVMediaTypeAudio is valid for this API, and AVFoundation copies the
        // completion block before invoking it on an arbitrary dispatch queue.
        unsafe {
            AVCaptureDevice::requestAccessForMediaType_completionHandler(media_type, &completion)
        };
        receiver
            .recv_timeout(Duration::from_secs(60))
            .map_err(|_| CommandError {
                code: "microphone_permission_timeout".to_string(),
                message: "macOS did not finish the microphone permission request.".to_string(),
            })?;
        Ok::<(), CommandError>(())
    })
    .await
    .map_err(|error| CommandError {
        code: "microphone_permission_cancelled".to_string(),
        message: error.to_string(),
    })?
}

#[cfg(not(target_os = "macos"))]
pub async fn request_microphone() -> Result<(), CommandError> {
    Err(CommandError {
        code: "unsupported_platform".to_string(),
        message: "Microphone permission requests are only available on macOS.".to_string(),
    })
}

pub fn list() -> Vec<SystemPermission> {
    vec![
        SystemPermission {
            id: "microphone",
            status: microphone_status(),
        },
        SystemPermission {
            id: "accessibility",
            status: if accessibility_granted() {
                "granted".to_string()
            } else {
                "denied".to_string()
            },
        },
        SystemPermission {
            id: "screen",
            status: if screen_recording_granted() {
                "granted".to_string()
            } else {
                "denied".to_string()
            },
        },
    ]
}

#[cfg(target_os = "macos")]
pub fn open_settings(permission_id: &str) -> Result<(), CommandError> {
    let anchor = match permission_id {
        "microphone" => "Privacy_Microphone",
        "accessibility" => "Privacy_Accessibility",
        "screen" => "Privacy_ScreenCapture",
        _ => {
            return Err(CommandError {
                code: "invalid_permission".to_string(),
                message: format!("Unknown system permission: {permission_id}"),
            })
        }
    };
    std::process::Command::new("open")
        .arg(format!(
            "x-apple.systempreferences:com.apple.preference.security?{anchor}"
        ))
        .spawn()
        .map(|_| ())
        .map_err(|error| CommandError {
            code: "settings_unavailable".to_string(),
            message: error.to_string(),
        })
}

#[cfg(not(target_os = "macos"))]
pub fn open_settings(_permission_id: &str) -> Result<(), CommandError> {
    Err(CommandError {
        code: "unsupported_platform".to_string(),
        message: "System permission settings are only available on macOS.".to_string(),
    })
}

#[cfg(target_os = "macos")]
pub fn request_screen_recording() {
    core_graphics::access::ScreenCaptureAccess.request();
}

#[cfg(not(target_os = "macos"))]
pub fn request_screen_recording() {}
