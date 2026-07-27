//! Window hide/show helpers that work around a Windows WebView2 quirk.
//!
//! On Windows, `Window::hide()` has a known issue with WebView2 where a
//! window hidden through the close-button -> `prevent_close` -> `hide()`
//! flow (i.e. the main/settings window being sent to the tray) cannot be
//! brought back by `show()` from the tray icon — the webview stays stuck
//! and the window never reappears.
//!
//! We work around this by minimizing the window and removing it from the
//! taskbar. Visually this is equivalent to `hide()` (no taskbar entry, the
//! window is not visible) but, unlike `hide()`, a minimized window can be
//! reliably restored via `unminimize()`.
//!
//! On macOS / Linux, `hide()` works correctly and is preferred: minimizing
//! would send the window to the Dock / taskbar, which is *not* the same as
//! hiding it.
//!
//! Always pair [`hide_window`] with [`show_window`] — never mix them with
//! raw `hide()` / `show()`, otherwise the skip-taskbar state on Windows can
//! get out of sync.

use tauri::WebviewWindow;

/// Hide a window so it can be reliably restored later by [`show_window`].
pub fn hide_window(w: &WebviewWindow) -> tauri::Result<()> {
    #[cfg(target_os = "windows")]
    {
        // Order matters: take it off the taskbar *before* minimizing so the
        // minimize doesn't briefly flash a taskbar entry.
        w.set_skip_taskbar(true)?;
        w.minimize()?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        w.hide()?;
    }
    Ok(())
}

/// Restore a window previously hidden with [`hide_window`] and focus it.
pub fn show_window(w: &WebviewWindow) -> tauri::Result<()> {
    #[cfg(target_os = "windows")]
    {
        w.set_skip_taskbar(false)?;
        w.unminimize()?;
        w.show()?;
        w.set_focus()?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        w.show()?;
        w.set_focus()?;
    }
    Ok(())
}
