use serde::{Deserialize, Serialize};

use crate::api::ccswitch_reader;
use crate::api::models::NormalizedUsage;
use crate::api::usage_aggregate::aggregate_usage;

/// 用量 API 响应
#[derive(Debug, Serialize, Deserialize)]
pub struct UsageApiResponse {
    pub success: bool,
    pub data: Option<NormalizedUsage>,
    pub error: Option<String>,
    pub record_count: usize,
    pub data_source: String,
}

/// 查询用量（从 CCSwitch 数据库聚合）
///
/// `data_source`：
/// - `"opencode"`（默认）— 只看 `_opencode_session` provider
/// - `"claude"` — 看 CCSwitch 里所有 claude 类型 provider
#[tauri::command]
pub async fn get_usage(data_source: Option<String>) -> Result<UsageApiResponse, String> {
    let ds = data_source.unwrap_or_else(|| "opencode".to_string());
    match ccswitch_reader::read_all_records() {
        Ok(all_records) => {
            let records = ccswitch_reader::filter_by_data_source(all_records, &ds);
            let total = records.len();
            let usage = aggregate_usage(&records);
            let has_data = usage.models.iter().any(|m| m.total_tokens > 0 || m.cost > 0.0)
                || usage.daily.iter().any(|d| d.total_tokens > 0 || d.cost > 0.0);

            Ok(UsageApiResponse {
                success: has_data,
                data: Some(usage),
                error: if has_data { None } else { Some("暂无用量记录".to_string()) },
                record_count: total,
                data_source: ds,
            })
        }
        Err(e) => Ok(UsageApiResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
            record_count: 0,
            data_source: ds,
        }),
    }
}

/// 获取所有原始记录（调试用）
#[derive(Debug, Serialize, Deserialize)]
pub struct RawRecordsResponse {
    pub records: Vec<crate::api::claude_log_reader::TokenUsageRecord>,
    pub error: Option<String>,
    pub data_source: String,
}

#[tauri::command]
pub async fn get_raw_records(data_source: Option<String>) -> RawRecordsResponse {
    let ds = data_source.unwrap_or_else(|| "opencode".to_string());
    match ccswitch_reader::read_all_records() {
        Ok(all) => {
            let records = ccswitch_reader::filter_by_data_source(all, &ds);
            RawRecordsResponse {
                records,
                error: None,
                data_source: ds,
            }
        }
        Err(e) => RawRecordsResponse {
            records: vec![],
            error: Some(e.to_string()),
            data_source: ds,
        },
    }
}
