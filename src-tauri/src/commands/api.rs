use serde::{Deserialize, Serialize};

use crate::api::ccswitch_reader;
use crate::api::claude_log_reader::TokenUsageRecord;

/// 用量 API 响应（始终返回全量原始记录，由前端按 dataSource 过滤）
#[derive(Debug, Serialize, Deserialize)]
pub struct UsageApiResponse {
    pub success: bool,
    pub raw_records: Vec<TokenUsageRecord>,
    pub error: Option<String>,
    pub record_count: usize,
}

/// 查询用量：拉取 CCSwitch 数据库中所有 provider 的原始记录
#[tauri::command]
pub async fn get_usage() -> Result<UsageApiResponse, String> {
    match ccswitch_reader::read_all_records() {
        Ok(raw_records) => {
            let total = raw_records.len();
            let has_data = !raw_records.is_empty();
            Ok(UsageApiResponse {
                success: has_data,
                raw_records,
                error: if has_data { None } else { Some("暂无用量记录".to_string()) },
                record_count: total,
            })
        }
        Err(e) => Ok(UsageApiResponse {
            success: false,
            raw_records: vec![],
            error: Some(e.to_string()),
            record_count: 0,
        }),
    }
}

/// 获取所有原始记录（调试用）
#[derive(Debug, Serialize, Deserialize)]
pub struct RawRecordsResponse {
    pub records: Vec<TokenUsageRecord>,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn get_raw_records() -> RawRecordsResponse {
    match ccswitch_reader::read_all_records() {
        Ok(records) => RawRecordsResponse {
            records,
            error: None,
        },
        Err(e) => RawRecordsResponse {
            records: vec![],
            error: Some(e.to_string()),
        },
    }
}
