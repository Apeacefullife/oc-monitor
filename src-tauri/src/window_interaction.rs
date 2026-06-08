//! 窗口交互：锁定后鼠标穿透桌面，标题栏按钮区仍可点击解锁。

use tauri::WebviewWindow;

use crate::window_style;

static INTERACTION_LOCKED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

const TITLEBAR_HEIGHT_LOGICAL: f64 = 36.0;
const TITLEBAR_ACTIONS_WIDTH_LOGICAL: f64 = 116.0;

pub fn is_locked() -> bool {
    INTERACTION_LOCKED.load(std::sync::atomic::Ordering::Relaxed)
}

pub fn titlebar_actions_width_logical() -> f64 {
    TITLEBAR_ACTIONS_WIDTH_LOGICAL
}

pub fn titlebar_height_logical() -> f64 {
    TITLEBAR_HEIGHT_LOGICAL
}

pub fn set_interaction_locked(window: &WebviewWindow, locked: bool) -> Result<(), String> {
    INTERACTION_LOCKED.store(locked, std::sync::atomic::Ordering::Relaxed);
    update_scale(window);

    #[cfg(target_os = "windows")]
    window_style::ensure_wndproc_hook(window)?;

    #[cfg(not(target_os = "windows"))]
    {
        window
            .set_ignore_cursor_events(locked)
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub fn update_scale(window: &WebviewWindow) {
    window_style::update_scale(window);
}
