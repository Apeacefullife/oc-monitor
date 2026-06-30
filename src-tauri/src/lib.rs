use tauri::Manager;

mod api;
mod auto_start;
mod commands;
mod store;
mod tray;
mod window_style;

/// 退出应用
#[tauri::command]
fn quit_app(app_handle: tauri::AppHandle) {
    app_handle.exit(0);
}

/// 显示主窗口
#[tauri::command]
fn show_main_window(app_handle: tauri::AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.set_skip_taskbar(false);
        window_style::prepare_show(&window);
        let _ = window.show();
        let _ = window.set_focus();
        window_style::apply_deferred(window);
        let _ = tray::refresh_quick_menu(&app_handle);
    }
}

/// 布局过渡期间暂停 resize 触发的磨玻璃刷新
#[tauri::command]
fn set_layout_transitioning(active: bool) {
    window_style::set_layout_transitioning(active);
}

/// 重刷窗口磨玻璃
#[tauri::command]
fn refresh_window_glass(app_handle: tauri::AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        window_style::apply_maintain(&window);
    }
}

/// 窗口定位到屏幕右下角
#[tauri::command]
fn position_window_bottom_right(app_handle: tauri::AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        position_main_window(&window);
    }
}

#[cfg(target_os = "windows")]
fn position_main_window(window: &tauri::WebviewWindow) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, SWP_NOACTIVATE, SWP_NOZORDER, SWP_NOSIZE,
    };

    if let Ok(hwnd_raw) = window.hwnd() {
        unsafe {
            let hwnd = HWND(hwnd_raw.0 as *mut _);
            let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
            let mut mi: MONITORINFO = std::mem::zeroed();
            mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;

            if GetMonitorInfoW(monitor, &mut mi).as_bool() {
                let wa = mi.rcWork;
                if let Ok(size) = window.outer_size() {
                    let x = ((wa.right - wa.left) - size.width as i32).max(0);
                    let y = ((wa.bottom - wa.top) - size.height as i32).max(0);
                    let _ = SetWindowPos(
                        hwnd, None, x, y, 0, 0,
                        SWP_NOZORDER | SWP_NOSIZE | SWP_NOACTIVATE,
                    );
                }
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn position_main_window(_window: &tauri::WebviewWindow) {}

/// 隐藏主窗口
#[tauri::command]
fn hide_main_window(app_handle: tauri::AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.set_skip_taskbar(true);
        let _ = window.hide();
        let _ = tray::refresh_quick_menu(&app_handle);
    }
}

/// 切换主窗口显隐
#[tauri::command]
fn toggle_main_window(app_handle: tauri::AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.set_skip_taskbar(true);
            let _ = window.hide();
        } else {
            let _ = window.set_skip_taskbar(false);
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
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            let _ = app.get_webview_window("main").map(|window| {
                let _ = window.show();
                let _ = window.set_focus();
            });
        }))
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_skip_taskbar(true);
                window_style::apply_force(&window);
                window_style::start_cursor_bounds_watchdog(app.handle().clone(), "main");
                // 定位到右下角再展示
                position_main_window(&window);
                window_style::apply_deferred(window.clone());
                window_style::start_watchdog(window.clone());
                let _ = window.set_skip_taskbar(false);
                let _ = window.show();
                let _ = window.set_focus();
            }

            tray::setup(app)?;

            // 后台静默轮询（从 CCSwitch 数据库读取用量）
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
            // 点击窗口外部自动隐藏
            if let tauri::WindowEvent::Focused(false) = event {
                if let Some(w) = window.app_handle().get_webview_window("main") {
                    let _ = w.hide();
                    let _ = tray::refresh_quick_menu(&window.app_handle());
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            quit_app,
            show_main_window,
            hide_main_window,
            toggle_main_window,
            refresh_window_glass,
            set_layout_transitioning,
            position_window_bottom_right,
            get_version,
            commands::api::get_usage,
            commands::api::get_raw_records,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
