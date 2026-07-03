/// CCSwitch SQLite 数据库读取器
///
/// 从 ~/.cc-switch/cc-switch.db 的 proxy_request_logs 表读取用量数据。
/// 自动定位当前激活的 provider（如 OpenCode），只显示对应数据。

use std::path::PathBuf;

use chrono::{TimeZone, Utc};
use rusqlite::Connection;

use super::claude_log_reader::TokenUsageRecord;
use super::pricing;

/// CCSwitch 数据库路径
fn db_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "无法获取 home 目录".to_string())?;
    Ok(home.join(".cc-switch").join("cc-switch.db"))
}

/// 获取指定 app_type 下当前激活的 provider_id
fn active_provider_id(conn: &Connection, app_type: &str) -> Result<Option<String>, String> {
    let mut stmt = conn
        .prepare("SELECT id FROM providers WHERE app_type = ?1 AND is_current = 1 LIMIT 1")
        .map_err(|e| format!("查询 provider 失败: {e}"))?;
    let result = stmt
        .query_row([app_type], |row| row.get(0))
        .ok();
    Ok(result)
}

/// 从 CCSwitch 读取所有 provider 的用量记录
///
/// 不再硬编码过滤某一种 provider：拉全 `proxy_request_logs` 表的记录，
/// 每条 record 携带 `provider_id` 字段，前端或上层按 dataSource 选项
/// 自行过滤（OpenCode 模式只看 `_opencode_session`，Claude 模式看 claude provider）。
pub fn read_all_records() -> Result<Vec<TokenUsageRecord>, String> {
    let path = db_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }

    let conn = Connection::open(&path)
        .map_err(|e| format!("打开 CCSwitch 数据库失败: {e}"))?;

    let mut stmt = conn
        .prepare(
            "SELECT created_at, model, input_tokens, output_tokens,
                    cache_read_tokens, cache_creation_tokens, total_cost_usd, provider_id
             FROM proxy_request_logs
             ORDER BY created_at ASC",
        )
        .map_err(|e| format!("查询准备失败: {e}"))?;

    let records = stmt
        .query_map([], |row| {
            let ts: i64 = row.get(0)?;
            let model: String = row.get(1)?;
            let input_tokens: i64 = row.get(2)?;
            let output_tokens: i64 = row.get(3)?;
            let cache_read: i64 = row.get(4)?;
            let cache_creation: i64 = row.get(5)?;
            let _cost_usd: String = row.get(6)?;
            let provider_id: String = row.get(7).unwrap_or_else(|_| String::new());

            let input = input_tokens.max(0) as u64;
            let output = output_tokens.max(0) as u64;
            let cache_r = cache_read.max(0) as u64;
            let cache_c = cache_creation.max(0) as u64;

            // 过滤 thinking/空消息（0 输入 0 输出），避免请求数虚增
            if input == 0 && output == 0 && cache_r == 0 && cache_c == 0 {
                return Ok(None);
            }

            Ok(Some(TokenUsageRecord {
                timestamp: Utc.timestamp_opt(ts, 0).unwrap(),
                model: model.clone(),
                input_tokens: input,
                cache_read_tokens: cache_r,
                cache_creation_tokens: cache_c,
                output_tokens: output,
                total_tokens: input + output,
                request_count: 1,
                cost: pricing::calculate_cost(&model, input, cache_r, cache_c, output),
                provider_id,
            }))
        })
        .map_err(|e| format!("查询执行失败: {e}"))?
        .filter_map(|r| r.ok().flatten())
        .collect::<Vec<_>>();

    Ok(records)
}

/// 按 dataSource 过滤 CCSwitch 记录
///
/// - `"opencode"`：只看 `_opencode_session` provider
/// - `"claude"`：看 provider_id 含 "claude" 的所有 provider
///   （兼容 CCSwitch 多种命名：`claude`、`_claude_session`、`claude-code` 等）
/// - 其他 / 默认：等同于 `"opencode"`
pub fn filter_by_data_source(records: Vec<TokenUsageRecord>, data_source: &str) -> Vec<TokenUsageRecord> {
    match data_source {
        "claude" => records
            .into_iter()
            .filter(|r| r.provider_id.to_lowercase().contains("claude"))
            .collect(),
        _ => records
            .into_iter()
            .filter(|r| r.provider_id == "_opencode_session")
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_path() {
        let path = db_path().unwrap();
        assert!(path.ends_with(".cc-switch/cc-switch.db"));
    }
}
