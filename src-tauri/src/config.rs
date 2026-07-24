use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

pub const STORE_FILE: &str = "settings.json";
pub const DEFAULT_INTERVAL_MIN: u32 = 30;
pub const DEFAULT_AUTOSTART: bool = false;
pub const DEFAULT_INTENSITY: u32 = 5;

#[derive(Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub interval_min: u32,
    pub autostart: bool,
    pub intensity: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            interval_min: DEFAULT_INTERVAL_MIN,
            autostart: DEFAULT_AUTOSTART,
            intensity: DEFAULT_INTENSITY,
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

        Ok(Self {
            interval_min,
            autostart,
            intensity,
        })
    }

    pub fn save(&self, app: &AppHandle) -> Result<(), String> {
        let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
        store.set("interval_min", json!(self.interval_min));
        store.set("autostart", json!(self.autostart));
        store.set("intensity", json!(self.intensity));
        store.save().map_err(|e| e.to_string())?;
        Ok(())
    }
}
