//! Owns notch placement, global activation, and hover dismissal.
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager, Monitor, PhysicalPosition};

const WIDTH: f64 = 660.0;
const HOT_WIDTH: f64 = 280.0;
const HOT_HEIGHT: f64 = 44.0;
const SURFACE_WIDTH: f64 = 360.0;
const SURFACE_HEIGHT: f64 = 36.0;
const LEAVE_DELAY: Duration = Duration::from_millis(400);

#[derive(Clone, Copy, Debug)]
struct Bounds {
    x: f64,
    y: f64,
    width: f64,
    scale: f64,
}
impl From<&Monitor> for Bounds {
    fn from(m: &Monitor) -> Self {
        Self {
            x: m.position().x as f64,
            y: m.position().y as f64,
            width: m.size().width as f64,
            scale: m.scale_factor(),
        }
    }
}
impl Bounds {
    fn contains(&self, cursor: PhysicalPosition<f64>, width: f64, height: f64) -> bool {
        let dx = (cursor.x - self.x - self.width / 2.0) / self.scale;
        let dy = (cursor.y - self.y) / self.scale;
        dx.abs() <= width / 2.0 && dy >= 0.0 && dy <= height
    }
    fn window_position(&self) -> PhysicalPosition<i32> {
        PhysicalPosition::new(
            (self.x + (self.width - WIDTH * self.scale) / 2.0).round() as i32,
            self.y.round() as i32,
        )
    }
}

#[derive(Default)]
struct Activation {
    ready: bool,
    revealed: bool,
    pinned: bool,
    busy: bool,
    suppressed: bool,
    monitor: Option<Bounds>,
    left_at: Option<Instant>,
    ignores_cursor: bool,
    surface: Option<(f64, f64)>,
}

impl Activation {
    fn should_conceal(&mut self, now: Instant, inside: bool) -> bool {
        if !self.revealed || inside || self.pinned || self.busy {
            self.left_at = None;
            return false;
        }
        now.duration_since(*self.left_at.get_or_insert(now)) >= LEAVE_DELAY
    }
}

#[derive(Default)]
pub struct HudState {
    activation: Mutex<Activation>,
    pub stopped: AtomicBool,
}

fn reveal(
    app: &AppHandle,
    state: &mut Activation,
    monitor: Bounds,
    pinned: bool,
) -> Result<(), String> {
    let hud = app
        .get_webview_window("hud")
        .ok_or("Widget window unavailable")?;
    hud.set_position(monitor.window_position())
        .map_err(|e| e.to_string())?;
    hud.show().map_err(|e| e.to_string())?;
    if pinned {
        hud.set_focus().map_err(|e| e.to_string())?;
    }
    state.monitor = Some(monitor);
    state.pinned |= pinned;
    state.suppressed = false;
    state.left_at = None;
    if !state.revealed {
        hud.emit("hud-reveal", true).map_err(|e| e.to_string())?;
        state.revealed = true;
    }
    Ok(())
}

fn conceal(app: &AppHandle, state: &mut Activation) -> Result<(), String> {
    let hud = app
        .get_webview_window("hud")
        .ok_or("Widget window unavailable")?;
    // Keep the transparent webview mounted so an interrupted exit can reverse.
    hud.set_ignore_cursor_events(true)
        .map_err(|e| e.to_string())?;
    hud.emit("hud-reveal", false).map_err(|e| e.to_string())?;
    state.revealed = false;
    state.pinned = false;
    state.ignores_cursor = true;
    state.left_at = None;
    Ok(())
}

#[tauri::command]
pub async fn hud_ready(app: AppHandle, state: tauri::State<'_, HudState>) -> Result<bool, String> {
    let mut activation = state.activation.lock().map_err(|e| e.to_string())?;
    activation.ready = true;
    if !activation.revealed {
        let hud = app
            .get_webview_window("hud")
            .ok_or("Widget window unavailable")?;
        hud.set_ignore_cursor_events(true)
            .map_err(|e| e.to_string())?;
        activation.ignores_cursor = true;
    }
    Ok(activation.revealed)
}

#[tauri::command]
pub async fn show_hud(app: AppHandle, state: tauri::State<'_, HudState>) -> Result<(), String> {
    let cursor = app.cursor_position().map_err(|e| e.to_string())?;
    let monitor = app
        .monitor_from_point(cursor.x, cursor.y)
        .map_err(|e| e.to_string())?
        .or(app.primary_monitor().map_err(|e| e.to_string())?)
        .ok_or("No display available")?;
    let mut activation = state.activation.lock().map_err(|e| e.to_string())?;
    reveal(&app, &mut activation, Bounds::from(&monitor), true)
}

#[tauri::command]
pub async fn hide_hud(app: AppHandle, state: tauri::State<'_, HudState>) -> Result<(), String> {
    let mut activation = state.activation.lock().map_err(|e| e.to_string())?;
    conceal(&app, &mut activation)?;
    // Closing while still near the notch must not immediately reopen it.
    activation.suppressed = true;
    Ok(())
}

#[tauri::command]
pub async fn set_hud_busy(state: tauri::State<'_, HudState>, busy: bool) -> Result<(), String> {
    state.activation.lock().map_err(|e| e.to_string())?.busy = busy;
    Ok(())
}

// Keep native hover/click regions aligned with the rendered state; the rest of
// the transparent window must pass through to the application underneath.
#[tauri::command]
pub async fn set_hud_surface(
    state: tauri::State<'_, HudState>,
    surface: String,
) -> Result<(), String> {
    let size = match surface.as_str() {
        "compact" => (SURFACE_WIDTH, SURFACE_HEIGHT),
        "response" => (560.0, 112.0),
        "expanded" => (560.0, 360.0),
        _ => return Err("Unknown widget surface".into()),
    };
    state.activation.lock().map_err(|e| e.to_string())?.surface = Some(size);
    Ok(())
}

#[derive(Default)]
struct Shortcut {
    command_down: bool,
    chord_down: bool,
    last_command: Option<Instant>,
}
impl Shortcut {
    fn update(&mut self, now: Instant, command: bool, control: bool, option: bool) -> bool {
        let chord = control && option;
        let mut activate = chord && !self.chord_down;
        if command && !self.command_down && !control && !option {
            if self
                .last_command
                .is_some_and(|last| now.duration_since(last) <= Duration::from_millis(450))
            {
                activate = true;
                self.last_command = None;
            } else {
                self.last_command = Some(now);
            }
        }
        if control || option {
            self.last_command = None;
        }
        self.command_down = command;
        self.chord_down = chord;
        activate
    }
}

pub fn start(app: &AppHandle) {
    app.manage(HudState::default());
    #[cfg(target_os = "macos")]
    if let Some(hud) = app.get_webview_window("hud") {
        let native_hud = hud.clone();
        let _ = hud.run_on_main_thread(move || {
            if let Ok(pointer) = native_hud.ns_window() {
                // SAFETY: Tauri owns this live NSWindow and the closure runs on
                // the AppKit main thread. The reference never escapes the closure.
                let window = unsafe { &*pointer.cast::<objc2_app_kit::NSWindow>() };
                window.setLevel(objc2_app_kit::NSStatusWindowLevel + 1);
            }
        });
    }
    let app = app.clone();
    std::thread::spawn(move || {
        let mut shortcut = Shortcut::default();
        let mut monitors = Vec::new();
        let mut refreshed = Instant::now() - Duration::from_secs(2);
        loop {
            let state = app.state::<HudState>();
            if state.stopped.load(Ordering::Relaxed) {
                break;
            }
            let now = Instant::now();
            if refreshed.elapsed() >= Duration::from_secs(1) {
                if let Ok(current) = app.available_monitors() {
                    monitors = current;
                }
                refreshed = now;
            }
            #[cfg(target_os = "macos")]
            let triggered = {
                use objc2_app_kit::{NSEvent, NSEventModifierFlags as Flags};
                let flags = NSEvent::modifierFlags_class();
                shortcut.update(
                    now,
                    flags.contains(Flags::Command),
                    flags.contains(Flags::Control),
                    flags.contains(Flags::Option),
                )
            };
            #[cfg(not(target_os = "macos"))]
            let triggered = shortcut.update(now, false, false, false);

            if let (Ok(cursor), Ok(mut activation)) =
                (app.cursor_position(), state.activation.lock())
            {
                if activation.ready {
                    let hot = monitors
                        .iter()
                        .map(Bounds::from)
                        .find(|m| m.contains(cursor, HOT_WIDTH, HOT_HEIGHT));
                    let (width, height) = activation
                        .surface
                        .unwrap_or((SURFACE_WIDTH, SURFACE_HEIGHT));
                    let inside = activation
                        .monitor
                        .is_some_and(|m| m.contains(cursor, width, height));
                    if !inside && hot.is_none() {
                        activation.suppressed = false;
                    }
                    if triggered {
                        if let Ok(Some(monitor)) = app.monitor_from_point(cursor.x, cursor.y) {
                            if let Err(error) =
                                reveal(&app, &mut activation, Bounds::from(&monitor), true)
                            {
                                eprintln!("HUD activation: {error}");
                            }
                        }
                    } else if !activation.revealed && !activation.suppressed {
                        if let Some(monitor) = hot {
                            if let Err(error) = reveal(&app, &mut activation, monitor, false) {
                                eprintln!("HUD activation: {error}");
                            }
                        }
                    }
                    if activation.should_conceal(now, inside) {
                        if let Err(error) = conceal(&app, &mut activation) {
                            eprintln!("HUD dismissal: {error}");
                        }
                    }
                    let ignore = !activation.revealed || !inside;
                    if ignore != activation.ignores_cursor {
                        if let Some(hud) = app.get_webview_window("hud") {
                            if hud.set_ignore_cursor_events(ignore).is_ok() {
                                activation.ignores_cursor = ignore;
                            }
                        }
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(30));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn notch_regions_use_display_origin_and_retina_scale() {
        let monitor = Bounds {
            x: -3024.0,
            y: -1964.0,
            width: 3024.0,
            scale: 2.0,
        };
        assert_eq!(
            monitor.window_position(),
            PhysicalPosition::new(-2172, -1964)
        );
        assert!(monitor.contains(
            PhysicalPosition::new(-1512.0, -1890.0),
            HOT_WIDTH,
            HOT_HEIGHT
        ));
        assert!(!monitor.contains(
            PhysicalPosition::new(-1512.0, -1800.0),
            HOT_WIDTH,
            HOT_HEIGHT
        ));
        assert!(monitor.contains(
            PhysicalPosition::new(-1512.0, -1910.0),
            SURFACE_WIDTH,
            SURFACE_HEIGHT
        ));
        assert!(!monitor.contains(
            PhysicalPosition::new(-2000.0, -1950.0),
            HOT_WIDTH,
            HOT_HEIGHT
        ));
    }
    #[test]
    fn hover_exit_is_delayed_and_reentry_cancels_it() {
        let now = Instant::now();
        let mut state = Activation {
            revealed: true,
            ..Default::default()
        };
        assert!(!state.should_conceal(now, false));
        assert!(!state.should_conceal(now + Duration::from_millis(399), false));
        assert!(!state.should_conceal(now + Duration::from_millis(400), true));
        assert!(!state.should_conceal(now + Duration::from_millis(500), false));
        assert!(state.should_conceal(now + Duration::from_millis(900), false));
        state.pinned = true;
        assert!(!state.should_conceal(now + Duration::from_secs(3), false));
        state.pinned = false;
        state.busy = true;
        assert!(!state.should_conceal(now + Duration::from_secs(4), false));
        state.busy = false;
        assert!(!state.should_conceal(now + Duration::from_secs(5), false));
        assert!(state.should_conceal(now + Duration::from_millis(5400), false));
    }

    #[test]
    fn shortcuts_trigger_once_per_chord_or_double_tap() {
        let mut shortcut = Shortcut::default();
        let now = Instant::now();
        assert!(!shortcut.update(now, false, true, false));
        assert!(shortcut.update(now, false, true, true));
        assert!(!shortcut.update(now, false, true, true));
        assert!(!shortcut.update(now, false, false, false));
        assert!(!shortcut.update(now, true, false, false));
        assert!(!shortcut.update(now, true, false, false));
        assert!(!shortcut.update(now, false, false, false));
        assert!(shortcut.update(now + Duration::from_millis(200), true, false, false));
        assert!(!shortcut.update(now, false, false, false));
        assert!(!shortcut.update(now + Duration::from_secs(1), true, false, false));
    }
}
