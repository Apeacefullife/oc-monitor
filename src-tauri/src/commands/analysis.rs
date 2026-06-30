use tauri::AppHandle;

#[derive(Debug, serde::Serialize)]
pub struct AiAnalysisResponse {
    pub success: bool,
    pub content: Option<String>,
    pub error: Option<String>,
}

/// 本地用量分析：根据前端传来的用量数据生成简洁的分析报告
#[tauri::command]
pub async fn analyze_usage_ai(
    _app_handle: AppHandle,
    payload: String,
    locale: Option<String>,
) -> Result<AiAnalysisResponse, String> {
    let lang = locale.as_deref().unwrap_or("zh");

    let data: serde_json::Value =
        serde_json::from_str(&payload).map_err(|e| format!("解析 payload 失败: {e}"))?;

    let monthly_cost = data["monthly_cost"].as_f64().unwrap_or(0.0);
    let tokens_7d = data["tokens_7d"].as_f64().unwrap_or(0.0);
    let cost_7d = data["cost_7d"].as_f64().unwrap_or(0.0);
    let output_ratio = data["output_ratio"].as_f64().unwrap_or(0.0);

    let hit_rate_today = data["cache_hit_rate"]["today"].as_f64().unwrap_or(0.0);
    let hit_rate_7d = data["cache_hit_rate"]["last_7_days"].as_f64().unwrap_or(0.0);
    let hit_rate_month = data["cache_hit_rate"]["month"].as_f64().unwrap_or(0.0);

    // 提取本地洞察
    let local_insights: Vec<String> = data["local_insights"]
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    let avg_daily_tokens = if tokens_7d > 0f64 {
        tokens_7d / 7f64
    } else {
        0f64
    };

    if lang == "zh" {
        let mut lines = Vec::new();

        lines.push("📊 用量分析".to_string());
        lines.push(format!("本月消费: ${:.4}", monthly_cost));
        lines.push(format!("最近 7 天总 Token: {}", fmt_num(tokens_7d as u64)));
        lines.push(format!("最近 7 天总花费: ${:.4}", cost_7d));
        lines.push(format!("日均 Token: {}", fmt_num(avg_daily_tokens as u64)));
        lines.push(format!("输出占比: {:.1}%", output_ratio * 100.0));

        if tokens_7d > 0f64 {
            lines.push(String::new());
            lines.push("📈 缓存命中率".to_string());
            lines.push(format!("今日: {:.1}%", hit_rate_today));
            lines.push(format!("近 7 天: {:.1}%", hit_rate_7d));
            lines.push(format!("本月: {:.1}%", hit_rate_month));
        }

        if !local_insights.is_empty() {
            lines.push(String::new());
            lines.push("💡 小提示".to_string());
            for insight in &local_insights {
                lines.push(format!("· {}", insight));
            }
        }

        Ok(AiAnalysisResponse {
            success: true,
            content: Some(lines.join("\n")),
            error: None,
        })
    } else {
        let mut lines = Vec::new();

        lines.push("📊 Usage Analysis".to_string());
        lines.push(format!("Monthly cost: ${:.4}", monthly_cost));
        lines.push(format!("Last 7 days tokens: {}", fmt_num(tokens_7d as u64)));
        lines.push(format!("Last 7 days cost: ${:.4}", cost_7d));
        lines.push(format!("Daily avg tokens: {}", fmt_num(avg_daily_tokens as u64)));
        lines.push(format!("Output ratio: {:.1}%", output_ratio * 100.0));

        if tokens_7d > 0f64 {
            lines.push(String::new());
            lines.push("📈 Cache Hit Rate".to_string());
            lines.push(format!("Today: {:.1}%", hit_rate_today));
            lines.push(format!("Last 7 days: {:.1}%", hit_rate_7d));
            lines.push(format!("Month: {:.1}%", hit_rate_month));
        }

        if !local_insights.is_empty() {
            lines.push(String::new());
            lines.push("💡 Tips".to_string());
            for insight in &local_insights {
                lines.push(format!("· {}", insight));
            }
        }

        Ok(AiAnalysisResponse {
            success: true,
            content: Some(lines.join("\n")),
            error: None,
        })
    }
}

fn fmt_num(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.2}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}
