use serde::{Deserialize, Serialize};

use super::models::DailyUsageItem;
use chrono::Datelike;

/// 平台内部 API 统一响应：{ code, data, msg }
pub fn unwrap_platform_payload(json: &serde_json::Value) -> Result<serde_json::Value, String> {
    if let Some(code) = json.get("code").and_then(|c| c.as_i64()) {
        if code != 0 {
            let msg = json
                .get("msg")
                .and_then(|m| m.as_str())
                .unwrap_or("平台 API 错误");
            if code == 40002 {
                return Err(
                    "未登录或 Token 已过期：请在用量页窗口完成登录，确认页面能显示用量后再同步"
                        .to_string(),
                );
            }
            return Err(format!("{msg} (code {code})"));
        }
        if let Some(data) = json.get("data") {
            if data.is_null() {
                return Err("平台返回空数据".to_string());
            }
            return Ok(data.clone());
        }
    }
    Ok(json.clone())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyUsageSummary {
    pub total_cost: f64,
    pub currency: String,
    pub month: String,
    pub total_tokens: u64,
    pub request_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedUsage {
    pub daily: Vec<DailyUsageItem>,
    /// 按模型拆分的月度用量（供模型列表展示）
    pub models: Vec<DailyUsageItem>,
    pub monthly: MonthlyUsageSummary,
    pub has_daily_granularity: bool,
}

#[derive(Clone)]
struct DsEntry {
    date: Option<String>,
    model: String,
    usage: Vec<(String, f64)>,
}

const PROMPT_TYPES: &[&str] = &[
    "PROMPT_TOKEN",
    "PROMPT_CACHE_HIT_TOKEN",
    "PROMPT_CACHE_MISS_TOKEN",
];
const COMPLETION_TYPES: &[&str] = &["RESPONSE_TOKEN"];

fn is_cache_hit(t: &str) -> bool {
    if t == "PROMPT_CACHE_HIT_TOKEN" {
        return true;
    }
    let lower = t.to_ascii_lowercase();
    lower.contains("cache_hit") || lower.contains("cachehit")
}

fn is_cache_miss(t: &str) -> bool {
    if t == "PROMPT_CACHE_MISS_TOKEN" || t == "PROMPT_TOKEN" {
        return true;
    }
    let lower = t.to_ascii_lowercase();
    lower.contains("cache_miss") || lower.contains("cachemiss")
}

fn is_prompt(t: &str) -> bool {
    if is_cache_hit(t) || is_cache_miss(t) {
        return false;
    }
    if PROMPT_TYPES.contains(&t) {
        return true;
    }
    let lower = t.to_ascii_lowercase();
    lower.contains("prompt")
        || lower.contains("input_cache")
        || (lower.contains("input") && lower.contains("token"))
}

fn sync_input_totals(item: &mut DailyUsageItem) {
    if item.input_cache_hit_tokens > 0 || item.input_cache_miss_tokens > 0 {
        item.input_tokens = item.input_cache_hit_tokens + item.input_cache_miss_tokens;
    } else {
        item.input_cache_miss_tokens = item.input_tokens;
    }
    item.total_tokens = item.input_tokens + item.output_tokens;
}

fn is_completion(t: &str) -> bool {
    if COMPLETION_TYPES.contains(&t) {
        return true;
    }
    let lower = t.to_ascii_lowercase();
    lower.contains("response")
        || lower.contains("completion")
        || (lower.contains("output") && lower.contains("token"))
}

fn json_u64(v: &serde_json::Value) -> u64 {
    v.as_u64()
        .or_else(|| v.as_i64().map(|n| n.max(0) as u64))
        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        .unwrap_or(0)
}

fn json_f64(v: &serde_json::Value) -> f64 {
    v.as_f64()
        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        .unwrap_or(0.0)
}

fn first_json_u64<'a>(obj: &'a serde_json::Value, keys: &[&str]) -> u64 {
    keys.iter()
        .find_map(|k| obj.get(*k))
        .map(json_u64)
        .unwrap_or(0)
}

fn first_json_f64<'a>(obj: &'a serde_json::Value, keys: &[&str]) -> f64 {
    keys.iter()
        .find_map(|k| obj.get(*k))
        .map(json_f64)
        .unwrap_or(0.0)
}

fn extract_entries(node: &serde_json::Value) -> Vec<DsEntry> {
    let mut out = Vec::new();
    walk(node, None, &mut out);
    out
}

fn walk(node: &serde_json::Value, inherited_date: Option<&str>, out: &mut Vec<DsEntry>) {
    match node {
        serde_json::Value::Array(arr) => {
            for item in arr {
                walk(item, inherited_date, out);
            }
        }
        serde_json::Value::Object(map) => {
            if let (Some(model), Some(usage)) = (
                map.get("model").and_then(|v| v.as_str()),
                map.get("usage").and_then(|v| v.as_array()),
            ) {
                let usage_items: Vec<(String, f64)> = usage
                    .iter()
                    .filter_map(|u| {
                        let t = u.get("type")?.as_str()?;
                        let amount = u
                            .get("amount")
                            .and_then(|a| a.as_f64())
                            .or_else(|| {
                                u.get("amount")
                                    .and_then(|a| a.as_str())
                                    .and_then(|s| s.parse().ok())
                            })
                            .unwrap_or(0.0);
                        Some((t.to_string(), amount))
                    })
                    .collect();
                out.push(DsEntry {
                    date: inherited_date.map(str::to_string),
                    model: model.to_string(),
                    usage: usage_items,
                });
                return;
            }

            let date_candidate = ["date", "day", "ds", "report_date"]
                .iter()
                .find_map(|k| map.get(*k).and_then(|v| v.as_str()))
                .map(|s| s.chars().take(10).collect::<String>());

            let passed_date = date_candidate
                .as_deref()
                .or(inherited_date);

            for (k, v) in map {
                let next_date = if k == "total" {
                    Some("TOTAL")
                } else if is_date_key(k) {
                    Some(k.as_str())
                } else {
                    passed_date
                };
                walk(v, next_date, out);
            }
        }
        _ => {}
    }
}

fn find_currency(node: &serde_json::Value) -> Option<String> {
    match node {
        serde_json::Value::String(s) => {
            let v = s.trim().to_uppercase();
            if ["USD", "CNY", "RMB", "EUR", "JPY", "GBP", "HKD", "SGD"].contains(&v.as_str()) {
                Some(if v == "RMB" { "CNY".into() } else { v })
            } else {
                None
            }
        }
        serde_json::Value::Array(arr) => arr.iter().find_map(find_currency),
        serde_json::Value::Object(map) => {
            for key in ["currency", "settlement_currency", "settlementCurrency", "unit"] {
                if let Some(found) = map.get(key).and_then(find_currency) {
                    return Some(found);
                }
            }
            map.values().find_map(find_currency)
        }
        _ => None,
    }
}

fn is_date_key(d: &str) -> bool {
    d.len() == 10 && d.as_bytes().get(4) == Some(&b'-') && d.as_bytes().get(7) == Some(&b'-')
}

fn parse_usage_amounts(usage: &serde_json::Value) -> Vec<(String, f64)> {
    usage
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|u| {
                    let t = u.get("type")?.as_str()?;
                    let amount = u
                        .get("amount")
                        .and_then(|a| a.as_f64())
                        .or_else(|| {
                            u.get("amount")
                                .and_then(|a| a.as_str())
                                .and_then(|s| s.parse().ok())
                        })
                        .unwrap_or(0.0);
                    Some((t.to_string(), amount))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn find_days_array<'a>(node: &'a serde_json::Value) -> Option<&'a serde_json::Value> {
    match node {
        serde_json::Value::Object(map) => {
            if let Some(days) = map.get("days") {
                if days.is_array() || days.is_object() {
                    return Some(days);
                }
            }
            for value in map.values() {
                if let Some(found) = find_days_array(value) {
                    return Some(found);
                }
            }
            None
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                if let Some(found) = find_days_array(item) {
                    return Some(found);
                }
            }
            None
        }
        _ => None,
    }
}

fn day_models_array<'a>(day: &'a serde_json::Value) -> Option<&'a serde_json::Value> {
    ["data", "total", "usage", "models", "list", "items"]
        .iter()
        .find_map(|k| day.get(*k))
        .filter(|v| v.is_array())
}

fn apply_flat_day_fields(item: &mut DailyUsageItem, day: &serde_json::Value) {
    let cache_hit = first_json_u64(
        day,
        &[
            "prompt_cache_hit_tokens",
            "input_cache_hit_tokens",
            "cache_hit_tokens",
            "promptCacheHitTokens",
        ],
    );
    let cache_miss = first_json_u64(
        day,
        &[
            "prompt_cache_miss_tokens",
            "input_cache_miss_tokens",
            "cache_miss_tokens",
            "promptCacheMissTokens",
        ],
    );
    let input = first_json_u64(
        day,
        &[
            "prompt_tokens",
            "input_tokens",
            "inputTokens",
            "prompt_token",
            "input_token",
        ],
    );
    let output = first_json_u64(
        day,
        &[
            "completion_tokens",
            "output_tokens",
            "outputTokens",
            "response_token",
            "response_tokens",
        ],
    );
    let requests = first_json_u64(day, &["request_count", "requests", "request"]);
    let cost = first_json_f64(day, &["cost", "amount", "total_cost"]);

    if cache_hit > 0 || cache_miss > 0 {
        item.input_cache_hit_tokens += cache_hit;
        item.input_cache_miss_tokens += cache_miss;
    } else if input > 0 {
        item.input_cache_miss_tokens += input;
    }
    if output > 0 {
        item.output_tokens += output;
    }
    if requests > 0 {
        item.request_count += requests;
    }
    if cost > 0.0 {
        item.cost += cost;
    }
    sync_input_totals(item);
}

fn ingest_day_object(
    by_date: &mut std::collections::BTreeMap<String, DailyUsageItem>,
    day: &serde_json::Value,
    year: i32,
    month: u32,
) {
    let date = ["date", "utc_date", "day", "ds", "report_date"]
        .iter()
        .find_map(|k| day.get(*k))
        .and_then(|v| {
            if let Some(s) = v.as_str() {
                let trimmed = s.chars().take(10).collect::<String>();
                if is_date_key(&trimmed) {
                    return Some(trimmed);
                }
            }
            v.as_u64()
                .or_else(|| v.as_i64().map(|n| n as u64))
                .filter(|&d| (1..=31).contains(&d))
                .map(|d| format!("{year}-{month:02}-{d:02}"))
        });

    let Some(date) = date else {
        return;
    };

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    if date > today {
        return;
    }

    let item = by_date.entry(date.clone()).or_insert_with(|| DailyUsageItem {
        date,
        total_tokens: 0,
        input_tokens: 0,
        input_cache_hit_tokens: 0,
        input_cache_miss_tokens: 0,
        output_tokens: 0,
        request_count: 0,
        cost: 0.0,
        model: "all".to_string(),
    });

    if let Some(models) = day_models_array(day).and_then(|v| v.as_array()) {
        for model_entry in models {
            let usage_items = model_entry
                .get("usage")
                .map(parse_usage_amounts)
                .unwrap_or_default();
            accumulate_tokens(item, &usage_items);
        }
    }

    apply_flat_day_fields(item, day);
}

fn accumulate_tokens(item: &mut DailyUsageItem, usage: &[(String, f64)]) {
    for (t, amount) in usage {
        if t == "REQUEST" {
            item.request_count += *amount as u64;
        } else if is_cache_hit(t) {
            item.input_cache_hit_tokens += *amount as u64;
        } else if is_cache_miss(t) || is_prompt(t) {
            item.input_cache_miss_tokens += *amount as u64;
        } else if is_completion(t) {
            item.output_tokens += *amount as u64;
        }
    }
    sync_input_totals(item);
}

fn parse_year_month_hint(node: &serde_json::Value) -> Option<(i32, u32)> {
    let year = node.get("year").and_then(|v| v.as_i64())?;
    let month = node.get("month").and_then(|v| {
        v.as_u64().or_else(|| {
            v.as_str()
                .and_then(|s| s.split('-').nth(1))
                .and_then(|part| part.parse().ok())
        })
    })?;
    if (1..=12).contains(&(month as u32)) {
        Some((year as i32, month as u32))
    } else {
        None
    }
}

fn parse_daily_from_days(
    amount_json: &serde_json::Value,
    default_year: i32,
    default_month: u32,
) -> std::collections::BTreeMap<String, DailyUsageItem> {
    let mut by_date = std::collections::BTreeMap::new();
    let (year, month) =
        parse_year_month_hint(amount_json).unwrap_or((default_year, default_month));

    if let Some(days) = find_days_array(amount_json) {
        match days {
            serde_json::Value::Array(items) => {
                for day in items {
                    ingest_day_object(&mut by_date, day, year, month);
                }
            }
            serde_json::Value::Object(map) => {
                for (key, value) in map {
                    if is_date_key(key) {
                        ingest_day_object(&mut by_date, value, year, month);
                    } else if let serde_json::Value::Object(day_map) = value {
                        let mut merged = day_map.clone();
                        if !merged.contains_key("date") {
                            merged.insert(
                                "date".to_string(),
                                serde_json::Value::String(key.clone()),
                            );
                        }
                        ingest_day_object(
                            &mut by_date,
                            &serde_json::Value::Object(merged),
                            year,
                            month,
                        );
                    }
                }
            }
            _ => {}
        }
    }

    if by_date.is_empty() {
        for entry in extract_entries(amount_json) {
            let Some(date) = entry.date.as_ref().filter(|d| is_date_key(d)) else {
                continue;
            };
            let item = by_date.entry(date.clone()).or_insert_with(|| DailyUsageItem {
                date: date.clone(),
                total_tokens: 0,
                input_tokens: 0,
                input_cache_hit_tokens: 0,
                input_cache_miss_tokens: 0,
                output_tokens: 0,
                request_count: 0,
                cost: 0.0,
                model: "all".to_string(),
            });
            accumulate_tokens(item, &entry.usage);
        }
    }

    by_date
}

fn parse_daily_cost_from_days(cost_json: &serde_json::Value) -> std::collections::BTreeMap<String, f64> {
    let mut by_date = std::collections::BTreeMap::new();
    let Some(days) = find_days_array(cost_json).and_then(|v| v.as_array()) else {
        return by_date;
    };

    for day in days {
        let Some(date) = day
            .get("date")
            .and_then(|v| v.as_str())
            .filter(|d| is_date_key(d))
        else {
            continue;
        };
        let Some(models) = day_models_array(day).and_then(|v| v.as_array()) else {
            let flat = first_json_f64(day, &["cost", "amount", "total_cost"]);
            if flat > 0.0 {
                *by_date.entry(date.to_string()).or_insert(0.0) += flat;
            }
            continue;
        };

        let cost = by_date.entry(date.to_string()).or_insert(0.0);
        for model_entry in models {
            for (_, amount) in parse_usage_amounts(model_entry.get("usage").unwrap_or(&serde_json::Value::Null)) {
                *cost += amount;
            }
        }
    }

    by_date
}

fn merge_daily_map(
    target: &mut std::collections::BTreeMap<String, DailyUsageItem>,
    source: std::collections::BTreeMap<String, DailyUsageItem>,
) {
    for (date, item) in source {
        target.entry(date).or_insert(item);
    }
}

fn build_daily_map(
    daily_amount: Option<&serde_json::Value>,
    extra_daily_amount: Option<&serde_json::Value>,
    daily_cost: Option<&serde_json::Value>,
    year: i32,
    month: u32,
) -> std::collections::BTreeMap<String, DailyUsageItem> {
    let mut by_date = std::collections::BTreeMap::new();

    if let Some(src) = daily_amount {
        merge_daily_map(&mut by_date, parse_daily_from_days(src, year, month));
    }
    if let Some(src) = extra_daily_amount {
        let (y, m) = if month == 1 {
            (year - 1, 12)
        } else {
            (year, month - 1)
        };
        merge_daily_map(&mut by_date, parse_daily_from_days(src, y, m));
    }

    if let Some(cost) = daily_cost {
        for (date, cost) in parse_daily_cost_from_days(cost) {
            let item = by_date.entry(date.clone()).or_insert_with(|| DailyUsageItem {
                date: date.clone(),
                total_tokens: 0,
                input_tokens: 0,
                input_cache_hit_tokens: 0,
                input_cache_miss_tokens: 0,
                output_tokens: 0,
                request_count: 0,
                cost: 0.0,
                model: "all".to_string(),
            });
            item.cost += cost;
        }
    }

    by_date.retain(|_, item| item.total_tokens > 0 || item.cost > 0.0);
    by_date
}

/// 解析 DeepSeek 平台用量接口响应
pub fn normalize(
    amount_json: &serde_json::Value,
    cost_json: Option<&serde_json::Value>,
    daily_amount_json: Option<&serde_json::Value>,
    extra_amount_daily: Option<&serde_json::Value>,
) -> Option<NormalizedUsage> {
    let amount_entries = extract_entries(amount_json);
    if amount_entries.is_empty() {
        return None;
    }
    let cost_entries = cost_json.map(extract_entries).unwrap_or_default();

    let mut monthly_tokens: u64 = 0;
    let mut monthly_requests: u64 = 0;
    let mut monthly_cost = 0.0;

    let agg_src: Vec<_> = amount_entries
        .iter()
        .filter(|e| e.date.as_deref() == Some("TOTAL"))
        .collect();
    let agg_src = if agg_src.is_empty() {
        amount_entries
            .iter()
            .filter(|e| e.date.as_ref().is_none_or(|d| d == "TOTAL" || !is_date_key(d)))
            .collect()
    } else {
        agg_src
    };
    let agg_src = if agg_src.is_empty() {
        amount_entries.iter().collect()
    } else {
        agg_src
    };

    for e in &agg_src {
        let mut model_tokens = 0.0;
        for (t, amount) in &e.usage {
            if t == "REQUEST" {
                monthly_requests += *amount as u64;
            } else if is_prompt(t) || is_completion(t) {
                model_tokens += amount;
            }
        }
        monthly_tokens += model_tokens as u64;
        let _ = e.model.as_str();
    }

    let cost_agg: Vec<_> = cost_entries
        .iter()
        .filter(|e| e.date.as_deref() == Some("TOTAL"))
        .collect();
    let cost_agg = if cost_agg.is_empty() {
        cost_entries
            .iter()
            .filter(|e| e.date.as_ref().is_none_or(|d| d == "TOTAL" || !is_date_key(d)))
            .collect()
    } else {
        cost_agg
    };
    let cost_agg = if cost_agg.is_empty() {
        cost_entries.iter().collect()
    } else {
        cost_agg
    };

    for e in &cost_agg {
        for (_, amount) in &e.usage {
            monthly_cost += amount;
        }
    }

    let now = chrono::Local::now();
    let mut by_date = build_daily_map(
        daily_amount_json,
        extra_amount_daily,
        None,
        now.year(),
        now.month(),
    );

    if by_date.is_empty() {
        for e in &amount_entries {
            let Some(date) = e.date.as_ref().filter(|d| is_date_key(d)) else {
                continue;
            };
            let item = by_date.entry(date.clone()).or_insert_with(|| DailyUsageItem {
                date: date.clone(),
                total_tokens: 0,
                input_tokens: 0,
                input_cache_hit_tokens: 0,
                input_cache_miss_tokens: 0,
                output_tokens: 0,
                request_count: 0,
                cost: 0.0,
                model: "all".to_string(),
            });
            accumulate_tokens(item, &e.usage);
        }
        by_date.retain(|_, item| item.total_tokens > 0 || item.cost > 0.0);
    }

    for e in &cost_entries {
        let Some(date) = e.date.as_ref().filter(|d| is_date_key(d)) else {
            continue;
        };
        let item = by_date.entry(date.clone()).or_insert_with(|| DailyUsageItem {
            date: date.clone(),
            total_tokens: 0,
            input_tokens: 0,
            input_cache_hit_tokens: 0,
            input_cache_miss_tokens: 0,
            output_tokens: 0,
            request_count: 0,
            cost: 0.0,
            model: "all".to_string(),
        });
        for (_, amount) in &e.usage {
            item.cost += amount;
        }
    }

    let has_daily = !by_date.is_empty();
    let mut daily: Vec<DailyUsageItem> = by_date.into_values().collect();
    let models = build_model_usage(&amount_entries, &cost_entries);

    // 无日粒度时，用本月累计填充「今日」展示
    if !has_daily && (monthly_tokens > 0 || monthly_cost > 0.0) {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        daily.push(DailyUsageItem {
            date: today,
            total_tokens: monthly_tokens,
            input_tokens: 0,
            input_cache_hit_tokens: 0,
            input_cache_miss_tokens: 0,
            output_tokens: monthly_tokens,
            request_count: monthly_requests,
            cost: monthly_cost,
            model: "all".to_string(),
        });
    }

    let meaningful = monthly_tokens > 0 || monthly_requests > 0 || monthly_cost > 0.0 || has_daily;
    if !meaningful {
        return None;
    }

    let currency = cost_json
        .and_then(find_currency)
        .or_else(|| find_currency(amount_json))
        .unwrap_or_else(|| "CNY".to_string());

    Some(NormalizedUsage {
        daily,
        models,
        monthly: MonthlyUsageSummary {
            total_cost: monthly_cost,
            currency,
            month: now.format("%Y-%m").to_string(),
            total_tokens: monthly_tokens,
            request_count: monthly_requests,
        },
        has_daily_granularity: has_daily,
    })
}

fn build_model_usage(amount_entries: &[DsEntry], cost_entries: &[DsEntry]) -> Vec<DailyUsageItem> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let mut by_model: std::collections::BTreeMap<String, DailyUsageItem> =
        std::collections::BTreeMap::new();

    for e in amount_entries
        .iter()
        .filter(|e| e.date.as_deref() == Some("TOTAL") || e.date.as_ref().is_none_or(|d| !is_date_key(d)))
    {
        let item = by_model.entry(e.model.clone()).or_insert_with(|| DailyUsageItem {
            date: today.clone(),
            total_tokens: 0,
            input_tokens: 0,
            input_cache_hit_tokens: 0,
            input_cache_miss_tokens: 0,
            output_tokens: 0,
            request_count: 0,
            cost: 0.0,
            model: e.model.clone(),
        });
        for (t, amount) in &e.usage {
            if t == "REQUEST" {
                item.request_count += *amount as u64;
            } else if is_cache_hit(t) {
                item.input_cache_hit_tokens += *amount as u64;
            } else if is_cache_miss(t) || is_prompt(t) {
                item.input_cache_miss_tokens += *amount as u64;
            } else if is_completion(t) {
                item.output_tokens += *amount as u64;
            }
        }
        sync_input_totals(item);
    }

    for e in cost_entries
        .iter()
        .filter(|e| e.date.as_deref() == Some("TOTAL") || e.date.as_ref().is_none_or(|d| !is_date_key(d)))
    {
        if let Some(item) = by_model.get_mut(&e.model) {
            for (_, amount) in &e.usage {
                item.cost += amount;
            }
        }
    }

    by_model
        .into_values()
        .filter(|m| m.total_tokens > 0 || m.cost > 0.0)
        .collect()
}

async fn fetch_platform_json(
    client: &reqwest::Client,
    url: &str,
    cookie: Option<&str>,
    token: Option<&str>,
) -> Result<serde_json::Value, String> {
    let mut req = client
        .get(url)
        .header("Accept", "application/json")
        .header("Referer", "https://platform.deepseek.com/usage")
        .header("Origin", "https://platform.deepseek.com")
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        );
    if let Some(cookie) = cookie {
        req = req.header("Cookie", cookie);
    }
    if let Some(token) = token {
        req = req.header("Authorization", format!("Bearer {token}"));
    }

    let res = req
        .send()
        .await
        .map_err(|e| format!("用量请求失败: {e}"))?;

    if !res.status().is_success() {
        return Err(format!(
            "用量接口返回 HTTP {}，请重新登录平台",
            res.status()
        ));
    }

    let json: serde_json::Value = res
        .json()
        .await
        .map_err(|e| format!("用量数据解析失败: {e}"))?;

    unwrap_platform_payload(&json)
}

fn usage_amount_url(year: i32, month: u32, daily: bool) -> String {
    let mut url = format!(
        "https://platform.deepseek.com/api/v0/usage/amount?year={year}&month={month}"
    );
    if daily {
        url.push_str("&group_by=day");
    }
    url
}

fn usage_cost_url(year: i32, month: u32, daily: bool) -> String {
    let mut url = format!(
        "https://platform.deepseek.com/api/v0/usage/cost?year={year}&month={month}"
    );
    if daily {
        url.push_str("&group_by=day");
    }
    url
}

pub async fn fetch_usage(cookie: Option<&str>, token: Option<&str>) -> Result<NormalizedUsage, String> {
    if cookie.is_none() && token.is_none() {
        return Err("缺少平台登录凭证，请重新同步用量".to_string());
    }

    let now = chrono::Local::now();
    let year = now.year();
    let month = now.month();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;

    let amount_payload = fetch_platform_json(
        &client,
        &usage_amount_url(year, month, false),
        cookie,
        token,
    )
    .await?;

    let daily_amount = fetch_platform_json(
        &client,
        &usage_amount_url(year, month, true),
        cookie,
        token,
    )
    .await
    .ok();

    let cost_json = fetch_platform_json(
        &client,
        &usage_cost_url(year, month, false),
        cookie,
        token,
    )
    .await
    .ok();

    let extra_amount_daily = if now.day() <= 7 {
        let (prev_year, prev_month) = if month == 1 {
            (year - 1, 12)
        } else {
            (year, month - 1)
        };
        fetch_platform_json(
            &client,
            &usage_amount_url(prev_year, prev_month, true),
            cookie,
            token,
        )
        .await
        .ok()
    } else {
        None
    };

    normalize(
        &amount_payload,
        cost_json.as_ref(),
        daily_amount.as_ref(),
        extra_amount_daily.as_ref(),
    )
    .ok_or_else(|| "用量数据为空或格式已变更".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_days_array_for_trend() {
        let amount = serde_json::json!({
            "biz_data": {
                "total": [{
                    "model": "deepseek-v4-pro",
                    "usage": [
                        {"type": "PROMPT_CACHE_MISS_TOKEN", "amount": 1000},
                        {"type": "RESPONSE_TOKEN", "amount": 500}
                    ]
                }],
                "days": [
                    {
                        "date": "2026-06-01",
                        "data": [{
                            "model": "deepseek-v4-pro",
                            "usage": [
                                {"type": "PROMPT_CACHE_MISS_TOKEN", "amount": 100},
                                {"type": "RESPONSE_TOKEN", "amount": 50}
                            ]
                        }]
                    },
                    {
                        "date": "2026-06-02",
                        "data": [{
                            "model": "deepseek-v4-pro",
                            "usage": [
                                {"type": "PROMPT_CACHE_MISS_TOKEN", "amount": 200},
                                {"type": "RESPONSE_TOKEN", "amount": 80}
                            ]
                        }]
                    }
                ]
            }
        });

        let usage = normalize(&amount, None, Some(&amount), None).expect("normalized");
        assert!(usage.has_daily_granularity);
        assert_eq!(usage.daily.len(), 2);
        let day1 = usage.daily.iter().find(|d| d.date.ends_with("-01")).expect("day1");
        let day2 = usage.daily.iter().find(|d| d.date.ends_with("-02")).expect("day2");
        assert_eq!(day1.total_tokens, 150);
        assert_eq!(day2.total_tokens, 280);
        assert_eq!(usage.monthly.total_tokens, 1500);
    }

    #[test]
    fn parse_days_inside_biz_data_array() {
        let amount = serde_json::json!({
            "biz_data": [{
                "total": [{
                    "model": "deepseek-v4-flash",
                    "usage": [
                        {"type": "PROMPT_CACHE_MISS_TOKEN", "amount": 300},
                        {"type": "RESPONSE_TOKEN", "amount": 120}
                    ]
                }],
                "days": [{
                    "date": "2026-06-03",
                    "data": [{
                        "model": "deepseek-v4-flash",
                        "usage": [
                            {"type": "PROMPT_CACHE_MISS_TOKEN", "amount": 30},
                            {"type": "RESPONSE_TOKEN", "amount": 12}
                        ]
                    }]
                }]
            }]
        });

        let usage = normalize(&amount, None, Some(&amount), None).expect("normalized");
        assert!(usage.has_daily_granularity);
        assert_eq!(usage.daily.len(), 1);
        assert_eq!(usage.daily[0].total_tokens, 42);
    }
}
