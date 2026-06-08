use serde::{Deserialize, Serialize};

use crate::api::client::DeepSeekClient;
use crate::api::error::ApiError;
use crate::api::models::BalanceResponse;
use crate::api::platform_usage::NormalizedUsage;
use crate::api::usage_aggregate::aggregate_usage;

/// API 响应包装
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

/// 用量 API 响应（含官方接口不可用标记）
#[derive(Debug, Serialize, Deserialize)]
pub struct UsageApiResponse {
    pub success: bool,
    pub data: Option<NormalizedUsage>,
    pub error: Option<String>,
    pub unavailable: bool,
    pub status_code: Option<u16>,
    pub record_count: usize,
}

/// 余额信息
#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceInfo {
    pub total_balance: f64,
    pub granted_balance: f64,
    pub topped_up_balance: f64,
    pub currency: String,
    pub is_available: bool,
    pub status: String, // "normal" | "low" | "exhausted"
}

/// 每日用量
#[derive(Debug, Serialize, Deserialize)]
pub struct DailyUsage {
    pub date: String,
    pub total_tokens: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub request_count: u64,
    pub cost: f64,
    pub model: String,
}

/// 月度消费
#[derive(Debug, Serialize, Deserialize)]
pub struct MonthlyCost {
    pub total_cost: f64,
    pub currency: String,
    pub month: String,
}

async fn usage_currency(client: &DeepSeekClient) -> String {
    match client.get_balance().await {
        Ok(balance) => balance
            .primary()
            .map(|info| info.currency.clone())
            .unwrap_or_else(|| "CNY".to_string()),
        Err(_) => "CNY".to_string(),
    }
}

/// 查询账户余额
#[tauri::command]
pub async fn get_balance(api_key: String) -> Result<ApiResponse<BalanceInfo>, String> {
    let client = DeepSeekClient::new(&api_key);
    match client.get_balance().await {
        Ok(balance) => {
            let Some(info) = balance.primary() else {
                return Ok(ApiResponse {
                    success: false,
                    data: None,
                    error: Some("余额数据为空".to_string()),
                });
            };

            let total = BalanceResponse::parse_f64(&info.total_balance);
            let granted = BalanceResponse::parse_f64(&info.granted_balance);
            let topped_up = BalanceResponse::parse_f64(&info.topped_up_balance);

            Ok(ApiResponse {
                success: true,
                data: Some(BalanceInfo {
                    total_balance: total,
                    granted_balance: granted,
                    topped_up_balance: topped_up,
                    currency: info.currency.clone(),
                    is_available: balance.is_available,
                    status: if !balance.is_available || total <= 0.0 {
                        "exhausted".to_string()
                    } else if total < 5.0 {
                        "low".to_string()
                    } else {
                        "normal".to_string()
                    },
                }),
                error: None,
            })
        }
        Err(e) => Ok(ApiResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// 查询用量（官方 GET /v1/usage，聚合为面板数据）
#[tauri::command]
pub async fn get_usage(
    _app_handle: tauri::AppHandle,
    api_key: String,
    days: Option<u32>,
) -> Result<UsageApiResponse, String> {
    let days = days.unwrap_or(7);
    let client = DeepSeekClient::new(&api_key);
    let currency = usage_currency(&client).await;

    match client.fetch_usage(days).await {
        Ok(records) => {
            let usage = aggregate_usage(&records, &currency);
            let has_data = records.iter().any(|r| r.total_tokens > 0 || r.cost_in_cents > 0);
            Ok(UsageApiResponse {
                success: has_data,
                data: Some(usage),
                error: if has_data {
                    None
                } else {
                    Some("近期无用量记录".to_string())
                },
                unavailable: false,
                status_code: Some(200),
                record_count: records.len(),
            })
        }
        Err(ApiError::UsageEndpointUnavailable) => Ok(UsageApiResponse {
            success: false,
            data: None,
            error: Some(
                "官方 /v1/usage 接口不可用（HTTP 404），请登录 platform 同步用量"
                    .to_string(),
            ),
            unavailable: true,
            status_code: Some(404),
            record_count: 0,
        }),
        Err(e) => Ok(UsageApiResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
            unavailable: false,
            status_code: e.status_code(),
            record_count: 0,
        }),
    }
}

/// 查询每日用量（兼容旧命令）
#[tauri::command]
pub async fn get_daily_usage(
    api_key: String,
    days: u32,
) -> Result<ApiResponse<Vec<DailyUsage>>, String> {
    let client = DeepSeekClient::new(&api_key);
    let currency = usage_currency(&client).await;

    match client.fetch_usage(days).await {
        Ok(records) => {
            let usage = aggregate_usage(&records, &currency);
            Ok(ApiResponse {
                success: true,
                data: Some(
                    usage
                        .daily
                        .into_iter()
                        .map(|u| DailyUsage {
                            date: u.date,
                            total_tokens: u.total_tokens,
                            input_tokens: u.input_tokens,
                            output_tokens: u.output_tokens,
                            request_count: u.request_count,
                            cost: u.cost,
                            model: u.model,
                        })
                        .collect(),
                ),
                error: None,
            })
        }
        Err(e) => Ok(ApiResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// 查询月度消费（从官方用量记录聚合）
#[tauri::command]
pub async fn get_monthly_cost(
    api_key: String,
) -> Result<ApiResponse<MonthlyCost>, String> {
    let client = DeepSeekClient::new(&api_key);
    let currency = usage_currency(&client).await;

    match client.fetch_usage(31).await {
        Ok(records) => {
            let usage = aggregate_usage(&records, &currency);
            Ok(ApiResponse {
                success: true,
                data: Some(MonthlyCost {
                    total_cost: usage.monthly.total_cost,
                    currency: usage.monthly.currency,
                    month: usage.monthly.month,
                }),
                error: None,
            })
        }
        Err(e) => Ok(ApiResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}
