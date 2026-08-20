use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

pub const STORE_FILE: &str = "settings.json";
pub const DEFAULT_INTERVAL_MIN: u32 = 30;
pub const DEFAULT_AUTOSTART: bool = false;
pub const DEFAULT_INTENSITY: u32 = 5;
pub const DEFAULT_OVERLAY_MODE: OverlayMode = OverlayMode::Auto;
pub const DEFAULT_OVERLAY_DURATION_SEC: u32 = 5;
pub const OVERLAY_DURATION_MIN: u32 = 2;
pub const OVERLAY_DURATION_MAX: u32 = 30;

/// How the shake overlay is dismissed after the shake animation finishes.
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum OverlayMode {
    /// Auto-hide after `overlay_duration_sec`.
    Auto,
    /// Stay visible until the user clicks the close button.
    Manual,
}

impl Default for OverlayMode {
    fn default() -> Self {
        DEFAULT_OVERLAY_MODE
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub interval_min: u32,
    pub autostart: bool,
    pub intensity: u32,
    pub overlay_mode: OverlayMode,
    pub overlay_duration_sec: u32,
    /// Session pause: while true, the countdown keeps cycling but the
    /// reminder (notification + shake) is suppressed. Persisted so the
    /// settings switch reflects the last choice.
    pub paused: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            interval_min: DEFAULT_INTERVAL_MIN,
            autostart: DEFAULT_AUTOSTART,
            intensity: DEFAULT_INTENSITY,
            overlay_mode: DEFAULT_OVERLAY_MODE,
            overlay_duration_sec: DEFAULT_OVERLAY_DURATION_SEC,
            paused: false,
        }
    }
}

impl AppConfig {
    pub fn load(app: &AppHandle) -> Result<Self, String> {
        let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;

        let interval_min = store
            .get("interval_min")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .unwrap_or(DEFAULT_INTERVAL_MIN)
            .clamp(1, 180);

        let autostart = store
            .get("autostart")
            .and_then(|v| v.as_bool())
            .unwrap_or(DEFAULT_AUTOSTART);

        let intensity = store
            .get("intensity")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .unwrap_or(DEFAULT_INTENSITY)
            .clamp(1, 10);

        // Tolerate older stores that predate the overlay-mode settings: fall
        // back to defaults when the key is missing or malformed. `store.get`
        // returns an owned `JsonValue`, so we deserialize straight from it
        // (OverlayMode's serde handles "auto"/"manual") instead of borrowing.
        let overlay_mode = store
            .get("overlay_mode")
            .and_then(|v| serde_json::from_value::<OverlayMode>(v).ok())
            .unwrap_or(DEFAULT_OVERLAY_MODE);

        let overlay_duration_sec = store
            .get("overlay_duration_sec")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .unwrap_or(DEFAULT_OVERLAY_DURATION_SEC)
            .clamp(OVERLAY_DURATION_MIN, OVERLAY_DURATION_MAX);

        // Older stores predate `paused`; missing key falls back to false.
        let paused = store
            .get("paused")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(Self {
            interval_min,
            autostart,
            intensity,
            overlay_mode,
            overlay_duration_sec,
            paused,
        })
    }

    pub fn save(&self, app: &AppHandle) -> Result<(), String> {
        let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
        store.set("interval_min", json!(self.interval_min));
        store.set("autostart", json!(self.autostart));
        store.set("intensity", json!(self.intensity));
        store.set("overlay_mode", json!(self.overlay_mode));
        store.set("overlay_duration_sec", json!(self.overlay_duration_sec));
        store.set("paused", json!(self.paused));
        store.save().map_err(|e| e.to_string())?;
        Ok(())
    }
}
