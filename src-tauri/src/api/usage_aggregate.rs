use std::collections::BTreeMap;

use chrono::Local;

use super::claude_log_reader::TokenUsageRecord;
use super::models::{DailyUsageItem, MonthlyUsageSummary, NormalizedUsage};

/// 规范化模型名
fn canonical_model(name: &str) -> String {
    let n = name.trim().to_lowercase();
    if n.is_empty() {
        return "unknown".to_string();
    }
    // DeepSeek
    if n.contains("v4-pro") || n.contains("deepseek-reasoner") {
        return "deepseek-v4-pro".to_string();
    }
    if n.contains("v4-flash") || n.contains("deepseek-chat") {
        return "deepseek-v4-flash".to_string();
    }
    // Claude
    if n.contains("claude-sonnet-4-6") || n.contains("claude-sonnet-4") {
        return "claude-sonnet-4".to_string();
    }
    if n.contains("claude-opus-4") {
        return "claude-opus-4".to_string();
    }
    if n.contains("claude-haiku") {
        return "claude-haiku".to_string();
    }
    // OpenAI
    if n.contains("gpt-4o-mini") {
        return "gpt-4o-mini".to_string();
    }
    if n.contains("gpt-4o") {
        return "gpt-4o".to_string();
    }
    if n.contains("o1") {
        return "o1".to_string();
    }
    if n.contains("o3") {
        return "o3-mini".to_string();
    }
    // Gemini
    if n.contains("gemini-2.5-flash") {
        return "gemini-2.5-flash".to_string();
    }
    if n.contains("gemini-2.5-pro") || n.contains("gemini-2.5") {
        return "gemini-2.5-pro".to_string();
    }
    if n.contains("gemini-2.0") || n.contains("gemini-2") {
        return "gemini-2.0-flash".to_string();
    }
    // Llama
    if n.contains("llama-4") {
        return "llama-4".to_string();
    }
    if n.contains("llama") {
        return "llama-3".to_string();
    }
    // Qwen
    if n.contains("qwen") {
        return "qwen".to_string();
    }
    // Mistral
    if n.contains("mistral") || n.contains("mixtral") {
        return "mistral".to_string();
    }
    name.trim().to_string()
}

fn parse_date(ts: &chrono::DateTime<chrono::Utc>) -> String {
    ts.format("%Y-%m-%d").to_string()
}

/// 聚合 Claude Code 用量记录为面板展示结构
pub fn aggregate_usage(records: &[TokenUsageRecord]) -> NormalizedUsage {
    let now = Local::now();
    let today_str = now.format("%Y-%m-%d").to_string();

    let mut daily_map: BTreeMap<String, DailyUsageItem> = BTreeMap::new();
    let mut model_map: BTreeMap<String, DailyUsageItem> = BTreeMap::new();
    let mut month_cost: f64 = 0.0;
    let mut month_tokens: u64 = 0;
    let mut month_requests: u64 = 0;

    for record in records {
        let date_str = parse_date(&record.timestamp);
        let canonical = canonical_model(&record.model);

        let input_cache_miss = record
            .input_tokens
            .saturating_sub(record.cache_read_tokens)
            .saturating_sub(record.cache_creation_tokens);

        // 每日聚合
        let daily = daily_map.entry(date_str.clone()).or_insert_with(|| DailyUsageItem {
            date: date_str.clone(),
            total_tokens: 0,
            input_tokens: 0,
            input_cache_hit_tokens: 0,
            input_cache_miss_tokens: 0,
            output_tokens: 0,
            request_count: 0,
            cost: 0.0,
            model: "all".to_string(),
        });
        daily.total_tokens += record.total_tokens;
        daily.input_tokens += record.input_tokens;
        daily.input_cache_hit_tokens += record.cache_read_tokens;
        daily.input_cache_miss_tokens += input_cache_miss;
        daily.output_tokens += record.output_tokens;
        daily.request_count += record.request_count;
        daily.cost += record.cost;

        // 模型聚合
        let model_item = model_map.entry(canonical.clone()).or_insert_with(|| DailyUsageItem {
            date: today_str.clone(),
            total_tokens: 0,
            input_tokens: 0,
            input_cache_hit_tokens: 0,
            input_cache_miss_tokens: 0,
            output_tokens: 0,
            request_count: 0,
            cost: 0.0,
            model: canonical.clone(),
        });
        model_item.total_tokens += record.total_tokens;
        model_item.input_tokens += record.input_tokens;
        model_item.input_cache_hit_tokens += record.cache_read_tokens;
        model_item.input_cache_miss_tokens += input_cache_miss;
        model_item.output_tokens += record.output_tokens;
        model_item.request_count += record.request_count;
        model_item.cost += record.cost;

        // 月度统计
        if let Some(dt) = record.timestamp.format("%Y-%m").to_string().parse::<String>().ok() {
            let month_str = &dt;
            if *month_str == now.format("%Y-%m").to_string() {
                month_cost += record.cost;
                month_tokens += record.total_tokens;
                month_requests += record.request_count;
            }
        }
    }

    let daily: Vec<DailyUsageItem> = daily_map.into_values().collect();
    let models: Vec<DailyUsageItem> = model_map
        .into_values()
        .filter(|m| m.total_tokens > 0 || m.cost > 0.0)
        .collect();
    let has_daily = !daily.is_empty();

    NormalizedUsage {
        daily,
        models,
        monthly: MonthlyUsageSummary {
            total_cost: month_cost,
            currency: "USD".to_string(),
            month: now.format("%Y-%m").to_string(),
            total_tokens: month_tokens,
            request_count: month_requests,
        },
        has_daily_granularity: has_daily,
    }
}
