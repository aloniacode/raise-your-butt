use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition};

const TAU: f32 = std::f32::consts::TAU;

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
    let pos = mon.position();
    let size = mon.size();
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
    let base = mon.position();

    overlay
        .set_position(PhysicalPosition::new(base.x, base.y))
        .map_err(|e| e.to_string())?;
    overlay.show().map_err(|e| e.to_string())?;
    overlay.set_focus().map_err(|e| e.to_string())?;

    // Notify the overlay webview so its card can re-trigger the CSS shake.
    let _ = app.emit_to("overlay", "shake-start", intensity);

    let app2 = app.clone();
    std::thread::spawn(move || {
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
            std::thread::sleep(Duration::from_millis(step_ms));
        }

        // Reset position.
        if let Some(w) = app2.get_webview_window("overlay") {
            let _ = w.set_position(PhysicalPosition::new(base.x, base.y));
        }

        // Hold for a moment, then hide the overlay.
        std::thread::sleep(Duration::from_millis(2500));
        if let Some(w) = app2.get_webview_window("overlay") {
            let _ = w.hide();
        }
    });

    Ok(())
}
