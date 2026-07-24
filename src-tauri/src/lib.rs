mod commands;
mod config;
mod shake;
mod timer;
mod tray;

use std::sync::Mutex;

use tauri::Manager;
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

use crate::config::AppConfig;
use crate::timer::TimerHandle;

pub struct AppState {
    pub config: Mutex<AppConfig>,
    pub timer: TimerHandle,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            // 1. Load persisted config (fallback to defaults).
            let cfg = AppConfig::load(app.handle())?;

            // 2. Initialize shared state.
            let timer = TimerHandle::new(cfg.interval_min);
            app.manage(AppState {
                config: Mutex::new(cfg.clone()),
                timer,
            });

            // 3. Size the overlay window to the primary monitor before first show.
            shake::init_overlay_size(app.handle())?;

            // 4. Start the background reminder timer.
            timer::spawn(app.handle().clone());

            // 5. Build the tray icon.
            tray::setup(app)?;

            // 6. Apply autostart preference on launch.
            if cfg.autostart {
                let _ = app.autolaunch().enable();
            } else {
                let _ = app.autolaunch().disable();
            }

            // 7. Close-on-X for the settings window should HIDE, not exit.
            if let Some(w) = app.get_webview_window("settings") {
                let w2 = w.clone();
                w.on_window_event(move |e| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = e {
                        api.prevent_close();
                        let _ = w2.hide();
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::set_config,
            commands::trigger_shake,
            commands::test_shake,
            commands::close_overlay,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
