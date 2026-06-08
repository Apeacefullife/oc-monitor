use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Emitter};

use crate::commands::api::{get_balance, ApiResponse, BalanceInfo};
use crate::commands::cache;
use crate::commands::platform;
use crate::commands::settings;
use crate::commands::tray_cmd;

static SILENT_REFRESH_ACTIVE: AtomicBool = AtomicBool::new(false);

struct RefreshGuard;

impl Drop for RefreshGuard {
    fn drop(&mut self) {
        SILENT_REFRESH_ACTIVE.store(false, Ordering::SeqCst);
    }
}

/// 后台静默刷新：API 余额 + 隐藏 WebView 抓取平台用量
#[tauri::command]
pub async fn silent_refresh(app_handle: AppHandle) -> Result<bool, String> {
    if SILENT_REFRESH_ACTIVE
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Ok(false);
    }
    let _guard = RefreshGuard;
    let Some(api_key) = settings::load_api_key(&app_handle)? else {
        return Ok(false);
    };

    let balance_res: ApiResponse<BalanceInfo> = get_balance(api_key).await?;
    let balance_json = balance_res
        .data
        .and_then(|b| serde_json::to_value(b).ok());

    let usage = platform::silent_fetch_platform_usage(&app_handle).await;

    let daily = usage
        .as_ref()
        .map(|u| serde_json::to_value(&u.daily).unwrap_or_default());
    let models = usage
        .as_ref()
        .map(|u| serde_json::to_value(&u.models).unwrap_or_default());
    let monthly = usage.as_ref().map(|u| {
        serde_json::json!({
            "total_cost": u.monthly.total_cost,
            "currency": u.monthly.currency,
            "month": u.monthly.month,
            "request_count": u.monthly.request_count,
        })
    });
    let platform_usage = usage
        .as_ref()
        .and_then(|u| serde_json::to_value(u).ok());

    let _ = cache::save_cached_data(
        app_handle.clone(),
        balance_json.clone(),
        daily,
        models,
        monthly.clone(),
        platform_usage,
        Some(chrono::Local::now().to_rfc3339()),
    )
    .await;

    let balance_amount = balance_json
        .as_ref()
        .and_then(|v| v.get("total_balance"))
        .and_then(|v| v.as_f64());
    let bal_currency = balance_json
        .as_ref()
        .and_then(|v| v.get("currency"))
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let monthly_cost = monthly
        .as_ref()
        .and_then(|v| v.get("total_cost"))
        .and_then(|v| v.as_f64());
    let usage_currency = monthly
        .as_ref()
        .and_then(|v| v.get("currency"))
        .and_then(|v| v.as_str())
        .map(str::to_string);

    let _ = tray_cmd::update_tray_tooltip(
        app_handle.clone(),
        balance_amount,
        bal_currency,
        monthly_cost,
        usage_currency,
    );

    let _ = app_handle.emit(
        "silent-refresh-done",
        serde_json::json!({
            "balance": balance_json,
            "usage": usage,
        }),
    );

    Ok(true)
}
