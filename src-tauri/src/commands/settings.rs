use tauri::{AppHandle, Emitter, Manager};

use crate::api::client::DeepSeekClient;
use crate::store::crypto;

const REFRESH_INTERVALS: [u64; 4] = [30, 60, 120, 300];
const DEFAULT_REFRESH_INTERVAL: u64 = 60;

pub fn load_refresh_interval(app_handle: &AppHandle) -> u64 {
    let store = crate::store::get_store(app_handle);
    let parsed = match store.get("refresh_interval") {
        Some(serde_json::Value::String(s)) => s.parse().ok(),
        Some(serde_json::Value::Number(n)) => n.as_u64(),
        _ => None,
    };
    parsed
        .filter(|v| REFRESH_INTERVALS.contains(v))
        .unwrap_or(DEFAULT_REFRESH_INTERVAL)
}

/// 保存 API Key（先验证，再加密存储）
#[tauri::command]
pub async fn save_api_key(app_handle: AppHandle, api_key: String) -> Result<bool, String> {
    let trimmed = api_key.trim();
    if trimmed.is_empty() {
        return Err("API Key 不能为空".to_string());
    }

    let client = DeepSeekClient::new(trimmed);
    client
        .get_balance()
        .await
        .map_err(|e| e.to_string())?;

    let store = crate::store::get_store(&app_handle);
    let encrypted = crypto::encrypt(trimmed).map_err(|e| e.to_string())?;
    store.set("api_key", serde_json::Value::String(encrypted));
    store
        .save()
        .map_err(|e| format!("保存失败: {e}"))?;
    Ok(true)
}

/// 读取已保存的 API Key（供后台刷新使用）
pub fn load_api_key(app_handle: &AppHandle) -> Result<Option<String>, String> {
    let store = crate::store::get_store(app_handle);
    match store.get("api_key") {
        Some(serde_json::Value::String(encrypted)) => {
            let decrypted = crypto::decrypt(&encrypted).map_err(|e| e.to_string())?;
            Ok(Some(decrypted))
        }
        _ => Ok(None),
    }
}

/// 获取 API Key（解密）
#[tauri::command]
pub async fn get_api_key(app_handle: AppHandle) -> Result<Option<String>, String> {
    load_api_key(&app_handle)
}

/// 保存设置项
#[tauri::command]
pub async fn save_setting(app_handle: AppHandle, key: String, value: String) -> Result<bool, String> {
    let store = crate::store::get_store(&app_handle);
    store.set(&key, serde_json::Value::String(value));
    let _ = store.save();
    Ok(true)
}

/// 获取设置项
#[tauri::command]
pub async fn get_setting(app_handle: AppHandle, key: String) -> Result<Option<String>, String> {
    let store = crate::store::get_store(&app_handle);
    match store.get(&key) {
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

    let store = crate::store::get_store(&app_handle);
    store.set(
        "refresh_interval",
        serde_json::Value::String(secs.to_string()),
    );
    store
        .save()
        .map_err(|e| format!("保存失败: {e}"))?;
    Ok(true)
}

/// 查询开机自启（以注册表为准）
#[tauri::command]
pub fn get_auto_start() -> bool {
    crate::auto_start::is_enabled()
}

/// 设置开机自启（写入注册表并持久化偏好）
#[tauri::command]
pub async fn set_auto_start(app_handle: AppHandle, enabled: bool) -> Result<bool, String> {
    if enabled {
        crate::auto_start::enable()?;
    } else {
        crate::auto_start::disable()?;
    }

    let store = crate::store::get_store(&app_handle);
    store.set(
        "auto_start",
        serde_json::Value::String(if enabled { "true" } else { "false" }.into()),
    );
    store
        .save()
        .map_err(|e| format!("保存设置失败: {e}"))?;

    Ok(true)
}

/// 清除全部本地数据（API Key、平台登录、缓存、设置）
#[tauri::command]
pub async fn clear_all_data(app_handle: AppHandle) -> Result<bool, String> {
    for label in ["platform-login", "platform-silent"] {
        if let Some(window) = app_handle.get_webview_window(label) {
            let _ = window.close();
        }
    }

    let _ = crate::auto_start::disable();

    let store = crate::store::get_store(&app_handle);
    store.clear();
    store
        .save()
        .map_err(|e| format!("清除数据失败: {e}"))?;

    let _ = app_handle.emit("all-data-cleared", ());
    Ok(true)
}
