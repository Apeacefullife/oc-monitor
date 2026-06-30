use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::store;

/// 缓存数据
#[derive(Debug, Serialize, Deserialize)]
pub struct CachedData {
    pub daily_usage: Option<serde_json::Value>,
    pub model_usage: Option<serde_json::Value>,
    pub monthly_cost: Option<serde_json::Value>,
    pub platform_usage: Option<serde_json::Value>,
    pub last_updated: Option<String>,
}

/// 保存缓存数据
#[tauri::command]
pub async fn save_cached_data(
    app_handle: AppHandle,
    daily_usage: Option<serde_json::Value>,
    model_usage: Option<serde_json::Value>,
    monthly_cost: Option<serde_json::Value>,
    platform_usage: Option<serde_json::Value>,
    last_updated: Option<String>,
) -> Result<bool, String> {
    let store = store::get_store(&app_handle);

    if let Some(v) = daily_usage {
        store.set("cache_daily_usage", v);
    }
    if let Some(v) = model_usage {
        store.set("cache_model_usage", v);
    }
    if let Some(v) = monthly_cost {
        store.set("cache_monthly_cost", v);
    }
    if let Some(v) = platform_usage {
        store.set("cached_platform_usage", v);
    }
    if let Some(v) = last_updated {
        store.set("cache_last_updated", serde_json::Value::String(v));
    }

    store
        .save()
        .map_err(|e| format!("缓存保存失败: {e}"))?;
    Ok(true)
}

/// 获取缓存数据
#[tauri::command]
pub async fn get_cached_data(app_handle: AppHandle) -> Result<Option<CachedData>, String> {
    let store = store::get_store(&app_handle);

    let cached = CachedData {
        daily_usage: store.get("cache_daily_usage"),
        model_usage: store.get("cache_model_usage"),
        monthly_cost: store.get("cache_monthly_cost"),
        platform_usage: store.get("cached_platform_usage"),
        last_updated: store
            .get("cache_last_updated")
            .and_then(|v| v.as_str().map(|s| s.to_string())),
    };

    Ok(Some(cached))
}

/// 清除缓存
#[tauri::command]
pub async fn clear_cache(app_handle: AppHandle) -> Result<bool, String> {
    let store = store::get_store(&app_handle);
    store.delete("cache_daily_usage");
    store.delete("cache_model_usage");
    store.delete("cache_monthly_cost");
    store.delete("cached_platform_usage");
    store.delete("cache_last_updated");
    let _ = store.save();
    Ok(true)
}
