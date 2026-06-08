use serde::{Deserialize, Serialize};

/// DeepSeek 余额条目
#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceInfoItem {
    pub currency: String,
    pub total_balance: String,
    pub granted_balance: String,
    pub topped_up_balance: String,
}

/// DeepSeek 账户余额响应（GET /user/balance）
#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceResponse {
    pub is_available: bool,
    pub balance_infos: Vec<BalanceInfoItem>,
}

impl BalanceResponse {
    pub fn primary(&self) -> Option<&BalanceInfoItem> {
        self.balance_infos
            .iter()
            .find(|b| b.currency.eq_ignore_ascii_case("CNY"))
            .or_else(|| self.balance_infos.first())
    }

    pub fn parse_f64(value: &str) -> f64 {
        value.trim().parse().unwrap_or(0.0)
    }
}

fn deserialize_flexible_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::String(s) => Ok(s),
        serde_json::Value::Number(n) => Ok(n.to_string()),
        serde_json::Value::Null => Ok(String::new()),
        other => Err(Error::custom(format!("unexpected id type: {other}"))),
    }
}

fn deserialize_flexible_i64<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Number(n) => n
            .as_i64()
            .or_else(|| n.as_f64().map(|v| v.round() as i64))
            .ok_or_else(|| Error::custom("invalid number")),
        serde_json::Value::String(s) => s
            .replace(',', "")
            .trim()
            .parse::<f64>()
            .map(|v| v.round() as i64)
            .map_err(|e| Error::custom(e.to_string())),
        serde_json::Value::Null => Ok(0),
        _ => Err(Error::custom("unexpected cost type")),
    }
}

fn deserialize_flexible_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Number(n) => n
            .as_u64()
            .or_else(|| n.as_f64().map(|v| v.round() as u64))
            .ok_or_else(|| Error::custom("invalid number")),
        serde_json::Value::String(s) => s
            .replace(',', "")
            .trim()
            .parse::<f64>()
            .map(|v| v.round() as u64)
            .map_err(|e| Error::custom(e.to_string())),
        serde_json::Value::Null => Ok(0),
        _ => Ok(0),
    }
}

/// 官方用量记录（GET /v1/usage）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    #[serde(default, deserialize_with = "deserialize_flexible_id")]
    pub id: String,
    #[serde(rename = "model_name", alias = "model", default)]
    pub model_name: String,
    #[serde(rename = "total_tokens", default, deserialize_with = "deserialize_flexible_u64")]
    pub total_tokens: u64,
    #[serde(
        rename = "prompt_tokens",
        default,
        deserialize_with = "deserialize_flexible_u64"
    )]
    pub prompt_tokens: u64,
    #[serde(
        rename = "input_cache_hit_tokens",
        default,
        deserialize_with = "deserialize_flexible_u64"
    )]
    pub input_cache_hit_tokens: u64,
    #[serde(
        rename = "input_cache_miss_tokens",
        default,
        deserialize_with = "deserialize_flexible_u64"
    )]
    pub input_cache_miss_tokens: u64,
    #[serde(
        rename = "completion_tokens",
        default,
        deserialize_with = "deserialize_flexible_u64"
    )]
    pub completion_tokens: u64,
    #[serde(
        rename = "cost_in_cents",
        alias = "cost",
        default,
        deserialize_with = "deserialize_flexible_i64"
    )]
    pub cost_in_cents: i64,
    #[serde(alias = "utc_date", default)]
    pub date: String,
    #[serde(
        rename = "request_count",
        default,
        deserialize_with = "deserialize_flexible_u64"
    )]
    pub request_count: u64,
}

/// 官方用量响应
#[derive(Debug, Deserialize)]
pub struct UsageResponse {
    pub data: Vec<UsageRecord>,
}

/// 每日用量条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyUsageItem {
    pub date: String,
    #[serde(rename = "total_tokens")]
    pub total_tokens: u64,
    #[serde(rename = "input_tokens", default)]
    pub input_tokens: u64,
    #[serde(rename = "input_cache_hit_tokens", default)]
    pub input_cache_hit_tokens: u64,
    #[serde(rename = "input_cache_miss_tokens", default)]
    pub input_cache_miss_tokens: u64,
    #[serde(rename = "output_tokens", default)]
    pub output_tokens: u64,
    #[serde(rename = "request_count", default)]
    pub request_count: u64,
    #[serde(default)]
    pub cost: f64,
    #[serde(default)]
    pub model: String,
}

/// 月度消费响应
#[derive(Debug, Serialize, Deserialize)]
pub struct MonthlyCostResponse {
    #[serde(rename = "total_cost")]
    pub total_cost: f64,
    #[serde(default = "default_currency")]
    pub currency: String,
    #[serde(default)]
    pub month: String,
}

fn default_currency() -> String {
    "CNY".to_string()
}
