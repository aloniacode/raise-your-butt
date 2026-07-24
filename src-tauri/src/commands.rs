use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::ManagerExt;

use crate::AppState;

#[derive(serde::Serialize)]
pub struct ConfigDto {
    pub interval_min: u32,
    pub autostart: bool,
    pub intensity: u32,
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
    }
}

#[tauri::command]
pub fn set_config(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    interval_min: Option<u32>,
    autostart: Option<bool>,
    intensity: Option<u32>,
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
    // so fiddling with intensity/autostart doesn't keep pushing the next
    // reminder back indefinitely.
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
