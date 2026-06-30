use tauri::AppHandle;

use crate::tray::{self, TrayQuickMenuLabels};

/// 更新托盘悬停提示（本月消费）
#[tauri::command]
pub fn update_tray_tooltip(
    app_handle: AppHandle,
    monthly_cost: Option<f64>,
) -> Result<bool, String> {
    let text = match monthly_cost {
        Some(m) if m > 0.0 => format!(
            "OC-Monitor\n本月 ${:.2}",
            m
        ),
        _ => "OC-Monitor".to_string(),
    };

    tray::update_tooltip(&app_handle, &text);
    Ok(true)
}

/// 同步托盘快捷操作台文案与状态
#[tauri::command]
pub fn sync_tray_quick_menu(
    app_handle: AppHandle,
    labels: TrayQuickMenuLabels,
) -> Result<bool, String> {
    tray::store_labels(&app_handle, labels)?;
    Ok(true)
}
