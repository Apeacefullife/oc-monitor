/// 系统托盘：快捷操作台菜单

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Emitter, Manager,
};

use crate::window_style;

type RuntimeMenuItem = MenuItem<tauri::Wry>;

#[derive(Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrayQuickMenuLabels {
    pub refresh: String,
    pub analysis: String,
    pub settings: String,
    pub show_window: String,
    pub hide_window: String,
    pub quit: String,
}

struct StoredLabels(std::sync::Mutex<Option<TrayQuickMenuLabels>>);

pub struct TrayQuickMenu {
    pub refresh: RuntimeMenuItem,
    pub analysis: RuntimeMenuItem,
    pub settings: RuntimeMenuItem,
    pub toggle_window: RuntimeMenuItem,
    pub quit: RuntimeMenuItem,
}

fn main_window_visible(app: &AppHandle) -> bool {
    app.get_webview_window("main")
        .and_then(|w| w.is_visible().ok())
        .unwrap_or(false)
}

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_skip_taskbar(false);
        window_style::prepare_show(&window);
        let _ = window.show();
        let _ = window.set_focus();
        window_style::apply_deferred(window);
    }
}

pub fn refresh_quick_menu(app: &AppHandle) -> Result<(), String> {
    let labels = app
        .try_state::<StoredLabels>()
        .and_then(|s| s.0.lock().ok().and_then(|g| g.clone()));

    let Some(labels) = labels else {
        return Ok(());
    };

    let menu = app
        .try_state::<TrayQuickMenu>()
        .ok_or("tray menu not initialized")?;

    let visible = main_window_visible(app);

    menu.refresh.set_text(&labels.refresh).map_err(|e| e.to_string())?;
    menu.analysis
        .set_text(&labels.analysis)
        .map_err(|e| e.to_string())?;
    menu.settings
        .set_text(&labels.settings)
        .map_err(|e| e.to_string())?;
    menu.toggle_window
        .set_text(if visible {
            &labels.hide_window
        } else {
            &labels.show_window
        })
        .map_err(|e| e.to_string())?;
    menu.quit.set_text(&labels.quit).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn store_labels(app: &AppHandle, labels: TrayQuickMenuLabels) -> Result<(), String> {
    if let Some(store) = app.try_state::<StoredLabels>() {
        if let Ok(mut guard) = store.0.lock() {
            *guard = Some(labels);
        }
    }
    refresh_quick_menu(app)
}

fn handle_menu_event(app: &AppHandle, id: &str) {
    match id {
        "refresh" => {
            let handle = app.clone();
            tauri::async_runtime::spawn(async move {
                let _ = crate::commands::refresh::silent_refresh(handle.clone()).await;
                let _ = handle.emit("tray-refresh-done", ());
            });
        }
        "analysis" => {
            let _ = app.emit("tray-open-analysis", ());
            show_main_window(app);
        }
        "settings" => {
            let _ = app.emit("tray-open-settings", ());
            show_main_window(app);
        }
        "toggle_window" => {
            if let Some(window) = app.get_webview_window("main") {
                if window.is_visible().unwrap_or(false) {
                    let _ = window.hide();
                } else {
                    show_main_window(app);
                }
                let _ = refresh_quick_menu(app);
            }
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    }
}

pub fn setup(app: &App) -> tauri::Result<()> {
    let refresh = MenuItem::with_id(app, "refresh", "刷新数据", true, None::<&str>)?;
    let analysis = MenuItem::with_id(app, "analysis", "AI 分析", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let toggle_window =
        MenuItem::with_id(app, "toggle_window", "隐藏窗口", true, None::<&str>)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, Some("CmdOrCtrl+Q"))?;

    let menu = Menu::with_items(
        app,
        &[
            &refresh,
            &analysis,
            &settings,
            &sep1,
            &toggle_window,
            &sep2,
            &quit,
        ],
    )?;

    app.manage(TrayQuickMenu {
        refresh,
        analysis,
        settings,
        toggle_window,
        quit,
    });
    app.manage(StoredLabels(std::sync::Mutex::new(None)));

    let _tray = TrayIconBuilder::with_id("oc-monitor-tray")
        .tooltip("OC-Monitor")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app_handle, event| {
            handle_menu_event(app_handle, event.id.as_ref());
        })
        .on_tray_icon_event(|tray_icon, event| {
            let app_handle = tray_icon.app_handle();

            if let TrayIconEvent::Click {
                button: MouseButton::Right,
                button_state: MouseButtonState::Down,
                ..
            } = event
            {
                let _ = refresh_quick_menu(app_handle);
            }

            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                if let Some(window) = app_handle.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        show_main_window(app_handle);
                    }
                    let _ = refresh_quick_menu(app_handle);
                }
            }
        })
        .build(app)?;

    Ok(())
}

/// 更新托盘图标提示文本
pub fn update_tooltip(app_handle: &AppHandle, text: &str) {
    if let Some(tray) = app_handle.tray_by_id("oc-monitor-tray") {
        let _ = tray.set_tooltip(Some(text));
    }
}
