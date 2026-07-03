/// Claude Code JSONL 日志读取器
///
/// 扫描 ~/.claude/projects/ 目录下的 JSONL 文件，
/// 解析 assistant 消息中的用量数据（tokens、模型、时间戳）。

use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::api::pricing;

/// 单条用量记录（从 assistant 消息提取）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsageRecord {
    /// 消息时间戳
    pub timestamp: DateTime<Utc>,
    /// 模型名
    pub model: String,
    /// 输入 tokens
    pub input_tokens: u64,
    /// 缓存命中 tokens
    pub cache_read_tokens: u64,
    /// 缓存创建 tokens
    pub cache_creation_tokens: u64,
    /// 输出 tokens
    pub output_tokens: u64,
    /// 总 tokens
    pub total_tokens: u64,
    /// 请求次数（JSONL 单条=1，CCSwitch 聚合行=实际次数）
    pub request_count: u64,
    /// 计算费用（USD）
    pub cost: f64,
    /// 来源 provider 标识（CCSwitch 用），JSONL 数据时为 "_claude_session"
    #[serde(default)]
    pub provider_id: String,
}

/// 查找 Claude Code 项目目录
fn claude_projects_dir() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "无法获取 home 目录".to_string())?;
    Ok(home.join(".claude").join("projects"))
}

/// 读取 ~/.claude/projects/ 下所有子目录中的 JSONL 文件并解析用量记录
pub fn read_all_records() -> Result<Vec<TokenUsageRecord>, String> {
    let projects_dir = claude_projects_dir()?;
    if !projects_dir.exists() {
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(&projects_dir)
        .map_err(|e| format!("读取目录失败 {}: {e}", projects_dir.display()))?;

    let mut all_records = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("读取条目失败: {e}"))?;
        let path = entry.path();
        if path.is_dir() {
            // 递归扫描子目录中的 JSONL 文件
            let sub_entries = fs::read_dir(&path)
                .map_err(|e| format!("读取子目录失败 {}: {e}", path.display()))?;
            for sub_entry in sub_entries {
                let sub_entry = sub_entry.map_err(|e| format!("读取子条目失败: {e}"))?;
                let sub_path = sub_entry.path();
                if sub_path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                    match read_records_from_file(&sub_path) {
                        Ok(records) => all_records.extend(records),
                        Err(e) => {
                            eprintln!("跳过文件 {}: {e}", sub_path.display());
                        }
                    }
                }
            }
        } else if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
            match read_records_from_file(&path) {
                Ok(records) => all_records.extend(records),
                Err(e) => {
                    eprintln!("跳过文件 {}: {e}", path.display());
                }
            }
        }
    }

    // 按时间戳排序
    all_records.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    Ok(all_records)
}

fn read_records_from_file(path: &PathBuf) -> Result<Vec<TokenUsageRecord>, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("读取文件失败: {e}"))?;

    let mut records = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(record) = try_parse_line(line) {
            records.push(record);
        }
    }

    Ok(records)
}

fn try_parse_line(line: &str) -> Option<TokenUsageRecord> {
    let json: serde_json::Value = serde_json::from_str(line).ok()?;

    // 只处理 assistant 类型消息
    if json.get("type")?.as_str()? != "assistant" {
        return None;
    }

    let message = json.get("message")?;
    let role = message.get("role")?.as_str()?;
    if role != "assistant" {
        return None;
    }

    let usage = message.get("usage")?;
    let model = message.get("model")?.as_str()?.to_string();

    let input_tokens = usage.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
    let cache_read = usage.get("cache_read_input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
    let cache_creation = usage.get("cache_creation_input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
    let output_tokens = usage.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0);

    // 过滤 thinking 消息（0 输入 0 输出），避免请求数翻倍
    if input_tokens == 0 && output_tokens == 0 {
        return None;
    }

    let total_tokens = input_tokens + output_tokens;

    let cost = pricing::calculate_cost(
        &model,
        input_tokens,
        cache_read,
        cache_creation,
        output_tokens,
    );

    // 解析时间戳
    let ts_str = json.get("timestamp").and_then(|v| v.as_str()).unwrap_or("");
    let timestamp = DateTime::parse_from_rfc3339(ts_str)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|| {
            // 尝试从 message 字段取时间
            message.get("timestamp")
                .and_then(|v| v.as_str())
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
        })
        .unwrap_or_else(|| {
            // 无时间戳则用文件修改时间，但这里给个兜底
            Utc::now()
        });

    Some(TokenUsageRecord {
        timestamp,
        model,
        input_tokens,
        cache_read_tokens: cache_read,
        cache_creation_tokens: cache_creation,
        output_tokens,
        total_tokens,
        request_count: 1,
        cost,
        provider_id: "_claude_session".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_assistant_line() {
        let line = r#"{"type":"assistant","message":{"role":"assistant","model":"deepseek-v4-flash","usage":{"input_tokens":100,"cache_read_input_tokens":20,"cache_creation_input_tokens":0,"output_tokens":50}},"timestamp":"2026-06-30T10:00:00Z"}"#;
        let record = try_parse_line(line).expect("should parse");
        assert_eq!(record.model, "deepseek-v4-flash");
        assert_eq!(record.input_tokens, 100);
        assert_eq!(record.cache_read_tokens, 20);
        assert_eq!(record.output_tokens, 50);
        assert_eq!(record.total_tokens, 150);
        assert!(record.cost > 0.0);
    }

    #[test]
    fn test_skip_user_message() {
        let line = r#"{"type":"user","message":{"role":"user","content":"hello"}}"#;
        assert!(try_parse_line(line).is_none());
    }
}
