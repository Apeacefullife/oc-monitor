use tauri::{AppHandle, Emitter};

use crate::store;

const REFRESH_INTERVALS: [u64; 4] = [30, 60, 120, 300];
const DEFAULT_REFRESH_INTERVAL: u64 = 60;

pub fn load_refresh_interval(app_handle: &AppHandle) -> u64 {
    let store_inst = store::get_store(app_handle);
    let parsed = match store_inst.get("refresh_interval") {
        Some(serde_json::Value::String(s)) => s.parse().ok(),
        Some(serde_json::Value::Number(n)) => n.as_u64(),
        _ => None,
    };
    parsed
        .filter(|v| REFRESH_INTERVALS.contains(v))
        .unwrap_or(DEFAULT_REFRESH_INTERVAL)
}

/// 保存设置项
#[tauri::command]
pub async fn save_setting(app_handle: AppHandle, key: String, value: String) -> Result<bool, String> {
    let store_inst = store::get_store(&app_handle);
    store_inst.set(&key, serde_json::Value::String(value));
    let _ = store_inst.save();
    Ok(true)
}

/// 获取设置项
#[tauri::command]
pub async fn get_setting(app_handle: AppHandle, key: String) -> Result<Option<String>, String> {
    let store_inst = store::get_store(&app_handle);
    match store_inst.get(&key) {
        Some(serde_json::Value::String(val)) => Ok(Some(val)),
        _ => Ok(None),
    }
}

/// 获取刷新间隔（秒）
#[tauri::command]
pub fn get_refresh_interval(app_handle: AppHandle) -> u64 {
    load_refresh_interval(&app_handle)
}

/// 保存刷新间隔（秒）
#[tauri::command]
pub async fn set_refresh_interval(app_handle: AppHandle, secs: u64) -> Result<bool, String> {
    if !REFRESH_INTERVALS.contains(&secs) {
        return Err(format!("不支持的刷新间隔: {secs}"));
    }

    let store_inst = store::get_store(&app_handle);
    store_inst.set(
        "refresh_interval",
        serde_json::Value::String(secs.to_string()),
    );
    store_inst
        .save()
        .map_err(|e| format!("保存失败: {e}"))?;
    Ok(true)
}

/// 查询开机自启
#[tauri::command]
pub fn get_auto_start() -> bool {
    crate::auto_start::is_enabled()
}

/// 设置开机自启
#[tauri::command]
pub async fn set_auto_start(app_handle: AppHandle, enabled: bool) -> Result<bool, String> {
    if enabled {
        crate::auto_start::enable()?;
    } else {
        crate::auto_start::disable()?;
    }

    let store_inst = store::get_store(&app_handle);
    store_inst.set(
        "auto_start",
        serde_json::Value::String(if enabled { "true" } else { "false" }.into()),
    );
    store_inst
        .save()
        .map_err(|e| format!("保存设置失败: {e}"))?;

    Ok(true)
}

/// 清除全部本地数据
#[tauri::command]
pub async fn clear_all_data(app_handle: AppHandle) -> Result<bool, String> {
    let _ = crate::auto_start::disable();

    let store_inst = store::get_store(&app_handle);
    store_inst.clear();
    store_inst
        .save()
        .map_err(|e| format!("清除数据失败: {e}"))?;

    let _ = app_handle.emit("all-data-cleared", ());
    Ok(true)
}
