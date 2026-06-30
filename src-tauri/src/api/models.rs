use serde::{Deserialize, Serialize};

/// 每日用量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyUsageItem {
    pub date: String,
    pub total_tokens: u64,
    pub input_tokens: u64,
    pub input_cache_hit_tokens: u64,
    pub input_cache_miss_tokens: u64,
    pub output_tokens: u64,
    pub request_count: u64,
    pub cost: f64,
    pub model: String,
}

/// 月度消费
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyUsageSummary {
    pub total_cost: f64,
    pub currency: String,
    pub month: String,
    pub total_tokens: u64,
    pub request_count: u64,
}

/// 归一化用量（前端面板展示结构）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedUsage {
    pub daily: Vec<DailyUsageItem>,
    pub models: Vec<DailyUsageItem>,
    pub monthly: MonthlyUsageSummary,
    pub has_daily_granularity: bool,
}
