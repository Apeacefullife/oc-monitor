use std::collections::BTreeMap;

use chrono::{Datelike, Local, NaiveDate};

use super::models::{DailyUsageItem, UsageRecord};
use super::platform_usage::{MonthlyUsageSummary, NormalizedUsage};

fn canonical_model(name: &str) -> String {
    let n = name.trim().to_lowercase();
    if n.is_empty() {
        return "unknown".to_string();
    }
    if n.contains("v4-pro") || n.contains("deepseek-reasoner") || n == "deepseek-pro" {
        return "deepseek-v4-pro".to_string();
    }
    if n.contains("v4-flash") || n.contains("deepseek-chat") || n == "deepseek-flash" {
        return "deepseek-v4-flash".to_string();
    }
    if n.contains("reasoner") || (n.contains("pro") && !n.contains("prompt")) {
        return "deepseek-v4-pro".to_string();
    }
    if n.contains("chat") || n.contains("flash") {
        return "deepseek-v4-flash".to_string();
    }
    name.trim().to_string()
}

fn parse_day(raw: &str) -> Option<NaiveDate> {
    let day = raw.chars().take(10).collect::<String>();
    NaiveDate::parse_from_str(&day, "%Y-%m-%d").ok()
}

/// 将官方 /v1/usage 记录聚合为面板展示结构
pub fn aggregate_usage(records: &[UsageRecord], currency: &str) -> NormalizedUsage {
    let now = Local::now();
    let today_str = now.format("%Y-%m-%d").to_string();

    let mut daily_map: BTreeMap<String, DailyUsageItem> = BTreeMap::new();
    let mut model_map: BTreeMap<String, DailyUsageItem> = BTreeMap::new();
    let mut month_cost_cents: i64 = 0;
    let mut month_tokens: u64 = 0;
    let mut month_requests: u64 = 0;

    for record in records {
        if record.model_name.trim().is_empty() && record.date.trim().is_empty() {
            continue;
        }

        let canonical = canonical_model(&record.model_name);
        let day_str = record.date.chars().take(10).collect::<String>();
        let input_tokens = record
            .input_cache_hit_tokens
            .saturating_add(record.input_cache_miss_tokens)
            .max(record.prompt_tokens);
        let cost = record.cost_in_cents as f64 / 100.0;

        let daily = daily_map
            .entry(day_str.clone())
            .or_insert_with(|| DailyUsageItem {
                date: day_str.clone(),
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
        daily.input_tokens += input_tokens;
        daily.input_cache_hit_tokens += record.input_cache_hit_tokens;
        daily.input_cache_miss_tokens += record
            .input_cache_miss_tokens
            .max(record.prompt_tokens.saturating_sub(record.input_cache_hit_tokens));
        daily.output_tokens += record.completion_tokens;
        daily.request_count += record.request_count;
        daily.cost += cost;

        let model_item = model_map
            .entry(canonical.clone())
            .or_insert_with(|| DailyUsageItem {
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
        model_item.input_tokens += input_tokens;
        model_item.input_cache_hit_tokens += record.input_cache_hit_tokens;
        model_item.input_cache_miss_tokens += record
            .input_cache_miss_tokens
            .max(record.prompt_tokens.saturating_sub(record.input_cache_hit_tokens));
        model_item.output_tokens += record.completion_tokens;
        model_item.request_count += record.request_count;
        model_item.cost += cost;

        if let Some(day) = parse_day(&record.date) {
            if day.year() == now.year() && day.month() == now.month() {
                month_cost_cents += record.cost_in_cents;
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
            total_cost: month_cost_cents as f64 / 100.0,
            currency: currency.to_string(),
            month: now.format("%Y-%m").to_string(),
            total_tokens: month_tokens,
            request_count: month_requests,
        },
        has_daily_granularity: has_daily,
    }
}
