use tauri::{Emitter, Manager};

mod api;
mod auto_start;
mod commands;
mod scheduler;
mod store;
mod tray;
mod window_style;
mod window_interaction;

/// 退出应用
#[tauri::command]
fn quit_app(app_handle: tauri::AppHandle) {
    app_handle.exit(0);
}

/// 显示主窗口
#[tauri::command]
fn show_main_window(app_handle: tauri::AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        window_style::prepare_show(&window);
        let _ = window.show();
        let _ = window.set_focus();
        window_style::apply_deferred(window);
        let _ = tray::refresh_quick_menu(&app_handle);
    }
}

/// 布局过渡期间暂停 resize 触发的磨玻璃刷新，避免侧边栏动画闪动
#[tauri::command]
fn set_layout_transitioning(active: bool) {
    window_style::set_layout_transitioning(active);
}

/// 锁定窗口交互：穿透桌面，仅标题栏按钮可点击；解锁后恢复
#[tauri::command]
fn set_window_interaction_locked(
    app_handle: tauri::AppHandle,
    locked: bool,
) -> Result<(), String> {
    let window = app_handle
        .get_webview_window("main")
        .ok_or("main window not found")?;
    window_interaction::set_interaction_locked(&window, locked)?;
    let _ = app_handle.emit("interaction-lock-changed", locked);
    let _ = tray::refresh_quick_menu(&app_handle);
    Ok(())
}

#[tauri::command]
fn is_window_interaction_locked() -> bool {
    window_interaction::is_locked()
}

/// 重刷窗口磨玻璃（前端在置顶切换等场景调用，轻量不闪动）
#[tauri::command]
fn refresh_window_glass(app_handle: tauri::AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        window_style::apply_maintain(&window);
    }
}

/// 隐藏主窗口
#[tauri::command]
fn hide_main_window(app_handle: tauri::AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.hide();
        let _ = tray::refresh_quick_menu(&app_handle);
    }
}

/// 切换主窗口显隐
#[tauri::command]
fn toggle_main_window(app_handle: tauri::AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            window_style::prepare_show(&window);
            let _ = window.show();
            let _ = window.set_focus();
            window_style::apply_deferred(window);
        }
    }
}

/// 获取版本号
#[tauri::command]
fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                window_style::apply_force(&window);
                window_style::start_cursor_bounds_watchdog(app.handle().clone(), "main");
                window_style::apply_deferred(window.clone());
                window_style::start_watchdog(window.clone());
                let _ = window.show();
                let _ = window.set_focus();
            }

            tray::setup(app)?;

            // 后台静默轮询（余额 + 平台用量，不弹窗）
            let bg = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                loop {
                    let _ = commands::refresh::silent_refresh(bg.clone()).await;
                    let secs = commands::settings::load_refresh_interval(&bg);
                    tokio::time::sleep(std::time::Duration::from_secs(secs)).await;
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            window_style::on_window_event(window, event);
        })
        .invoke_handler(tauri::generate_handler![
            quit_app,
            show_main_window,
            hide_main_window,
            toggle_main_window,
            refresh_window_glass,
            set_layout_transitioning,
            set_window_interaction_locked,
            is_window_interaction_locked,
            get_version,
            commands::api::get_balance,
            commands::api::get_usage,
            commands::api::get_daily_usage,
            commands::api::get_monthly_cost,
            commands::settings::save_api_key,
            commands::settings::get_api_key,
            commands::settings::save_setting,
            commands::settings::get_setting,
            commands::settings::get_auto_start,
            commands::settings::set_auto_start,
            commands::settings::get_refresh_interval,
            commands::settings::set_refresh_interval,
            commands::settings::clear_all_data,
            commands::cache::get_cached_data,
            commands::cache::save_cached_data,
            commands::cache::clear_cache,
            commands::tray_cmd::update_tray_tooltip,
            commands::tray_cmd::sync_tray_quick_menu,
            commands::refresh::silent_refresh,
            commands::analysis::analyze_usage_ai,
            commands::platform::has_platform_session,
            commands::platform::open_platform_login,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
