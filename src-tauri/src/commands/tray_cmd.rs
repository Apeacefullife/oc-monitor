use tauri::AppHandle;

use crate::tray::{self, TrayQuickMenuLabels};

/// 更新托盘悬停提示（余额 / 本月消费）
#[tauri::command]
pub fn update_tray_tooltip(
    app_handle: AppHandle,
    balance: Option<f64>,
    currency: Option<String>,
    monthly_cost: Option<f64>,
    usage_currency: Option<String>,
) -> Result<bool, String> {
    let bal_cur = currency.unwrap_or_else(|| "CNY".to_string());
    let use_cur = usage_currency.unwrap_or_else(|| bal_cur.clone());

    let text = match (balance, monthly_cost) {
        (Some(b), Some(m)) if m > 0.0 => format!(
            "DS-Monitor\n余额 {} {:.2}\n本月 {} {:.2}",
            symbol(&bal_cur),
            b,
            symbol(&use_cur),
            m
        ),
        (Some(b), _) => format!(
            "DS-Monitor\n余额 {} {:.2}",
            symbol(&bal_cur),
            b
        ),
        _ => "DS-Monitor".to_string(),
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

fn symbol(currency: &str) -> &'static str {
    match currency.to_uppercase().as_str() {
        "USD" => "$",
        "CNY" | "RMB" => "¥",
        "EUR" => "€",
        "GBP" => "£",
        "JPY" => "¥",
        _ => "",
    }
}
