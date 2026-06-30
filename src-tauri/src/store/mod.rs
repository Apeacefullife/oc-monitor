use std::sync::Arc;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

/// 获取应用 Store 实例
pub fn get_store(
    app_handle: &AppHandle,
) -> Arc<tauri_plugin_store::Store<tauri::Wry>> {
    app_handle.store("settings.json").expect("Failed to open settings store")
}
