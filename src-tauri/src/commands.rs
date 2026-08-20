use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::ManagerExt;

use crate::config::{OverlayMode, OVERLAY_DURATION_MAX, OVERLAY_DURATION_MIN};
use crate::AppState;

#[derive(serde::Serialize)]
pub struct ConfigDto {
    pub interval_min: u32,
    pub autostart: bool,
    pub intensity: u32,
    pub overlay_mode: OverlayMode,
    pub overlay_duration_sec: u32,
    pub paused: bool,
}

#[tauri::command]
pub fn get_config(state: tauri::State<'_, AppState>) -> ConfigDto {
    let c = state
        .config
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    ConfigDto {
        interval_min: c.interval_min,
        autostart: c.autostart,
        intensity: c.intensity,
        overlay_mode: c.overlay_mode,
        overlay_duration_sec: c.overlay_duration_sec,
        paused: c.paused,
    }
}

#[tauri::command]
pub fn set_config(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    interval_min: Option<u32>,
    autostart: Option<bool>,
    intensity: Option<u32>,
    overlay_mode: Option<String>,
    overlay_duration_sec: Option<u32>,
    paused: Option<bool>,
) -> Result<(), String> {
    // Update + persist + capture the new config while holding the lock briefly.
    let (new_cfg, interval_changed) = {
        let mut c = state
            .config
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let old_interval = c.interval_min;
        if let Some(v) = interval_min {
            c.interval_min = v.clamp(1, 180);
        }
        if let Some(v) = autostart {
            c.autostart = v;
        }
        if let Some(v) = intensity {
            c.intensity = v.clamp(1, 10);
        }
        if let Some(m) = overlay_mode {
            c.overlay_mode = match m.as_str() {
                "manual" => OverlayMode::Manual,
                _ => OverlayMode::Auto,
            };
        }
        if let Some(v) = overlay_duration_sec {
            c.overlay_duration_sec = v.clamp(OVERLAY_DURATION_MIN, OVERLAY_DURATION_MAX);
        }
        if let Some(v) = paused {
            c.paused = v;
        }
        c.save(&app)?;
        let changed = old_interval != c.interval_min;
        (c.clone(), changed)
    };

    // Sync autostart.
    let mgr = app.autolaunch();
    if new_cfg.autostart {
        let _ = mgr.enable();
    } else {
        let _ = mgr.disable();
    }

    // Only restart the timer countdown when the interval actually changed,
    // so fiddling with intensity/autostart/overlay settings doesn't keep
    // pushing the next reminder back indefinitely.
    if interval_changed {
        state.timer.set_interval(new_cfg.interval_min);
    }

    Ok(())
}

#[tauri::command]
pub fn trigger_shake(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let intensity = state
        .config
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .intensity;
    crate::shake::run_shake(&app, intensity)
}

#[tauri::command]
pub fn test_shake(app: AppHandle, intensity: u32) -> Result<(), String> {
    crate::shake::run_shake(&app, intensity.clamp(1, 10))
}

/// Hide the overlay window. Called from the overlay's close button when the
/// app is in manual overlay-dismiss mode.
#[tauri::command]
pub fn close_overlay(app: AppHandle) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("overlay") {
        let _ = w.hide();
    }
    Ok(())
}

/// Hide the settings window (close-to-tray) from the custom title-bar close
/// button. The settings window runs undecorated, so there is no native close
/// button; this mirrors what the old close-to-tray flow did.
#[tauri::command]
pub fn hide_settings(app: AppHandle) -> Result<(), String> {
    let w = app
        .get_webview_window("settings")
        .ok_or_else(|| "settings window not found".to_string())?;
    crate::window_util::hide_window(&w).map_err(|e| e.to_string())
}
