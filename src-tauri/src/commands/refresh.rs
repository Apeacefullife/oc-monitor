use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Emitter};

use crate::commands::cache;
use crate::commands::tray_cmd;
use crate::api::ccswitch_reader;
use crate::api::claude_log_reader;

static SILENT_REFRESH_ACTIVE: AtomicBool = AtomicBool::new(false);

struct RefreshGuard;

impl Drop for RefreshGuard {
    fn drop(&mut self) {
        SILENT_REFRESH_ACTIVE.store(false, Ordering::SeqCst);
    }
}

/// 后台静默刷新：合并两个数据源
///
/// 1. `ccswitch_reader` 拉 `~/.cc-switch/cc-switch.db` 的 `proxy_request_logs`，
///    覆盖 OpenCode / 其他 CCSwitch 代理的 provider
/// 2. `claude_log_reader` 拉 `~/.claude/projects/**/*.jsonl`，
///    覆盖**直接用 Claude Code CLI** 调任意 endpoint（DeepSeek / OpenCode Go / Anthropic），
///    这些调用只写 Claude Code 本地日志，不进 CCSwitch 数据库
///
/// 始终拉全量（不过滤）；前端按用户在设置里选的 dataSource
/// 对 `raw_records` 做过滤 + 聚合，实现"切换瞬时生效"。
#[tauri::command]
pub async fn silent_refresh(app_handle: AppHandle) -> Result<bool, String> {
    if SILENT_REFRESH_ACTIVE
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Ok(false);
    }
    let _guard = RefreshGuard;

    // 拉取 CCSwitch SQLite（OpenCode 等代理 provider 的记录）
    let mut raw_records: Vec<claude_log_reader::TokenUsageRecord> =
        match ccswitch_reader::read_all_records() {
            Ok(r) => r,
            Err(_) => Vec::new(),
        };

    // 追加拉取 Claude Code 本地 JSONL（直接用 Claude Code CLI 的记录）
    // JSONL 失败不阻塞 CCSwitch 数据
    match claude_log_reader::read_all_records() {
        Ok(mut jsonl_records) => raw_records.append(&mut jsonl_records),
        Err(_) => {}
    }

    if raw_records.is_empty() {
        return Ok(false);
    }

    // 保存缓存
    let raw_json = serde_json::to_value(&raw_records).ok();

    let _ = cache::save_cached_data(
        app_handle.clone(),
        None,             // daily_usage 暂存 None（前端不再读此字段）
        None,             // model_usage 同上
        None,             // monthly_cost 同上
        None,             // platform_usage 同上
        raw_json.clone(), // raw_records
        Some(chrono::Local::now().to_rfc3339()),
    )
    .await;

    // 更新托盘提示（dataSource 完全在前端，tray 先不区分）
    let _ = tray_cmd::update_tray_tooltip(
        app_handle.clone(),
        None,
    );

    // 发送事件到前端
    let _ = app_handle.emit(
        "silent-refresh-done",
        serde_json::json!({
            "raw_records": raw_json,
        }),
    );

    Ok(true)
}
