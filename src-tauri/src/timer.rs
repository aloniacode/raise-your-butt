use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;
use tokio::sync::Notify;

use crate::AppState;

pub struct TimerHandle {
    pub notify: Arc<Notify>,
    pub interval_min: AtomicU32,
}

impl TimerHandle {
    pub fn new(interval_min: u32) -> Self {
        Self {
            notify: Arc::new(Notify::new()),
            interval_min: AtomicU32::new(interval_min),
        }
    }

    pub fn set_interval(&self, min: u32) {
        self.interval_min.store(min, Ordering::Relaxed);
        // Wake the loop so it picks up the new interval immediately.
        self.notify.notify_one();
    }
}

pub fn spawn(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            // Hold `State` only long enough to read the interval and clone the
            // `Arc<Notify>`. After that the loop owns `notify` outright, so the
            // `notified()` future borrows an owned value rather than a temporary
            // `State` that would be dropped mid-await (E0716).
            let state = app.state::<AppState>();
            let mins = state.timer.interval_min.load(Ordering::Relaxed);
            let notify = state.timer.notify.clone();
            let dur = Duration::from_secs(mins as u64 * 60);

            tokio::select! {
                _ = notify.notified() => {
                    // Settings changed: the loop restarts with the new interval.
                }
                _ = tokio::time::sleep(dur) => {
                    fire(&app);
                }
            }
        }
    });
}

fn fire(app: &AppHandle) {
    // System notification.
    let _ = app
        .notification()
        .builder()
        .title("久坐提醒")
        .body("该起身活动啦！久坐伤身，起来走走吧")
        .show();

    // Screen shake with current intensity.
    let intensity = {
        app.state::<AppState>()
            .config
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .intensity
    };
    let _ = crate::shake::run_shake(app, intensity);
}
