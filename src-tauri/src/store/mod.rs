/// 本地持久化存储
///
/// 使用 tauri-plugin-store 实现：
/// - API Key（加密保存）
/// - 用量缓存（启动时快速恢复）
/// - 用户设置（刷新间隔、开机自启等）

pub mod crypto;

use std::sync::Arc;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

/// 获取应用 Store 实例
pub fn get_store(
    app_handle: &AppHandle,
) -> Arc<tauri_plugin_store::Store<tauri::Wry>> {
    app_handle.store("settings.json").expect("Failed to open settings store")
}
