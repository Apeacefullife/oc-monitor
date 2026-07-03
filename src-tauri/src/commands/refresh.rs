use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Emitter};

use crate::commands::cache;
use crate::commands::tray_cmd;
use crate::api::ccswitch_reader;
use crate::api::usage_aggregate::aggregate_usage;

static SILENT_REFRESH_ACTIVE: AtomicBool = AtomicBool::new(false);

struct RefreshGuard;

impl Drop for RefreshGuard {
    fn drop(&mut self) {
        SILENT_REFRESH_ACTIVE.store(false, Ordering::SeqCst);
    }
}

/// 后台静默刷新：从 CCSwitch 读取用量
///
/// `data_source`：`"opencode"`（默认）/`"claude"`
#[tauri::command]
pub async fn silent_refresh(
    app_handle: AppHandle,
    data_source: Option<String>,
) -> Result<bool, String> {
    if SILENT_REFRESH_ACTIVE
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Ok(false);
    }
    let _guard = RefreshGuard;
    let ds = data_source.unwrap_or_else(|| "opencode".to_string());

    // 读取所有记录，再按 dataSource 过滤
    let records = match ccswitch_reader::read_all_records() {
        Ok(all) => ccswitch_reader::filter_by_data_source(all, &ds),
        Err(_) => return Ok(false),
    };

    // 聚合用量
    let usage = aggregate_usage(&records);
    let usage_json = serde_json::to_value(&usage).ok();

    // 保存缓存
    let daily = usage_json.as_ref().and_then(|v| v.get("daily")).cloned();
    let models = usage_json.as_ref().and_then(|v| v.get("models")).cloned();
    let monthly = usage_json.as_ref().and_then(|v| v.get("monthly")).cloned();
    let platform_usage = usage_json.clone();

    let _ = cache::save_cached_data(
        app_handle.clone(),
        daily,
        models,
        monthly.clone(),
        platform_usage,
        Some(chrono::Local::now().to_rfc3339()),
    )
    .await;

    // 更新托盘提示
    let monthly_cost = monthly
        .as_ref()
        .and_then(|v| v.get("total_cost"))
        .and_then(|v| v.as_f64());

    let _ = tray_cmd::update_tray_tooltip(
        app_handle.clone(),
        monthly_cost,
    );

    // 发送事件到前端
    let _ = app_handle.emit(
        "silent-refresh-done",
        serde_json::json!({
            "usage": usage_json,
            "data_source": ds,
        }),
    );

    Ok(true)
}
