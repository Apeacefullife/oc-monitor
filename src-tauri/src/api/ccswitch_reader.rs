/// CCSwitch SQLite 数据库读取器
///
/// 从 ~/.cc-switch/cc-switch.db 的 proxy_request_logs 表读取用量数据。
/// 自动定位当前激活的 provider（如 OpenCode Go），只显示对应数据。

use std::path::PathBuf;

use chrono::{TimeZone, Utc};
use rusqlite::Connection;

use super::claude_log_reader::TokenUsageRecord;

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

/// 从 CCSwitch 读取用量记录（按当前激活的 provider 筛选）
///
/// 每条请求日志映射为一条 TokenUsageRecord，按时间升序排列。
pub fn read_all_records() -> Result<Vec<TokenUsageRecord>, String> {
    let path = db_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }

    let conn = Connection::open(&path)
        .map_err(|e| format!("打开 CCSwitch 数据库失败: {e}"))?;

    let pid = active_provider_id(&conn, "claude")?
        .ok_or_else(|| "未找到当前激活的 provider（claude）".to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT created_at, model, input_tokens, output_tokens,
                    cache_read_tokens, cache_creation_tokens, total_cost_usd
             FROM proxy_request_logs
             WHERE provider_id = ?1
             ORDER BY created_at ASC",
        )
        .map_err(|e| format!("查询准备失败: {e}"))?;

    let records = stmt
        .query_map([&pid], |row| {
            let ts: i64 = row.get(0)?;
            let model: String = row.get(1)?;
            let input_tokens: i64 = row.get(2)?;
            let output_tokens: i64 = row.get(3)?;
            let cache_read: i64 = row.get(4)?;
            let cache_creation: i64 = row.get(5)?;
            let cost_str: String = row.get(6)?;

            let input = input_tokens.max(0) as u64;
            let output = output_tokens.max(0) as u64;

            Ok(TokenUsageRecord {
                timestamp: Utc.timestamp_opt(ts, 0).unwrap(),
                model,
                input_tokens: input,
                cache_read_tokens: cache_read.max(0) as u64,
                cache_creation_tokens: cache_creation.max(0) as u64,
                output_tokens: output,
                total_tokens: input + output,
                request_count: 1,
                cost: cost_str.parse().unwrap_or(0.0),
            })
        })
        .map_err(|e| format!("查询执行失败: {e}"))?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>();

    Ok(records)
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
