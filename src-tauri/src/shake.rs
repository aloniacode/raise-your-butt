use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition};

use crate::config::OverlayMode;
use crate::AppState;

const TAU: f32 = std::f32::consts::TAU;

/// Payload pushed to the overlay webview when a shake starts. The frontend
/// uses `manual` to decide whether to render a close button.
#[derive(serde::Serialize, Clone)]
struct ShakeStart {
    intensity: u32,
    manual: bool,
}

/// Sizes and positions the overlay window. Tauri 2 `WebviewWindow` implements
/// `Manager`, so all window ops are available directly without any conversion.
pub fn init_overlay_size(app: &AppHandle) -> Result<(), String> {
    let w = app
        .get_webview_window("overlay")
        .ok_or_else(|| "overlay window not found".to_string())?;
    let mon = w
        .current_monitor()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "no monitor".to_string())?;
    // `position()` / `size()` borrow the monitor handle; deref to owned
    // `PhysicalPosition`/`PhysicalSize` (both `Copy`) so `mon` can drop freely.
    let pos = *mon.position();
    let size = *mon.size();
    w.set_position(PhysicalPosition::new(pos.x, pos.y))
        .map_err(|e| e.to_string())?;
    w.set_size(tauri::PhysicalSize::new(size.width, size.height))
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn offset_px(intensity: u32) -> i32 {
    ((intensity as f32) * 2.5).min(40.0) as i32
}

pub fn run_shake(app: &AppHandle, intensity: u32) -> Result<(), String> {
    let overlay = app
        .get_webview_window("overlay")
        .ok_or_else(|| "overlay window not found".to_string())?;
    let mon = overlay
        .current_monitor()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "no monitor".to_string())?;
    let base = *mon.position();

    overlay
        .set_position(PhysicalPosition::new(base.x, base.y))
        .map_err(|e| e.to_string())?;
    overlay.show().map_err(|e| e.to_string())?;
    overlay.set_focus().map_err(|e| e.to_string())?;

    // Read overlay dismiss settings so we know whether to auto-hide and how
    // long to hold, and tell the frontend whether to render a close button.
    let (manual, duration_sec) = {
        let state = app.state::<AppState>();
        let cfg = state
            .config
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        (
            cfg.overlay_mode == OverlayMode::Manual,
            cfg.overlay_duration_sec,
        )
    };

    let _ = app.emit_to(
        "overlay",
        "shake-start",
        ShakeStart { intensity, manual },
    );

    // Drive the 36-frame shake on a tokio task instead of a raw OS thread.
    // `set_position` is a quick synchronous window call; running it inside an
    // async task avoids dedicating a whole OS thread to an 800ms animation and
    // lets `tokio::time::sleep` yield cooperatively between frames.
    let app2 = app.clone();
    let handle = tauri::async_runtime::spawn(async move {
        let max = offset_px(intensity);
        let steps: u32 = 36;
        let total_ms: u64 = 800;
        let step_ms = (total_ms / steps as u64).max(1);

        for i in 0..steps {
            let t = i as f32 / steps as f32;
            let amp = (max as f32) * (1.0 - t);
            let dx = (amp * (t * TAU * 4.0).sin()) as i32;
            let dy = (amp * (t * TAU * 3.0).sin()) as i32;

            if let Some(w) = app2.get_webview_window("overlay") {
                let _ = w.set_position(PhysicalPosition::new(base.x + dx, base.y + dy));
            }
            tokio::time::sleep(Duration::from_millis(step_ms)).await;
        }

        // Settle back to the monitor's base position.
        if let Some(w) = app2.get_webview_window("overlay") {
            let _ = w.set_position(PhysicalPosition::new(base.x, base.y));
        }

        if manual {
            // Manual mode: leave the overlay on screen until the user clicks
            // the close button, which invokes the `close_overlay` command.
            return;
        }

        // Auto mode: hold for the configured duration, then hide.
        tokio::time::sleep(Duration::from_secs(duration_sec as u64)).await;
        if let Some(w) = app2.get_webview_window("overlay") {
            let _ = w.hide();
        }
    });

    // Track the animation task so a new shake can abort a still-running one,
    // preventing two animations from fighting over the window position.
    let state = app.state::<AppState>();
    let mut task = state
        .shake_task
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    if let Some(prev) = task.replace(handle) {
        prev.abort();
    }

    Ok(())
}
