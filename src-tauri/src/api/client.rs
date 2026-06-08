use chrono::Local;

use super::error::ApiError;
use super::models::{BalanceResponse, UsageRecord, UsageResponse};

/// DeepSeek API 客户端
pub struct DeepSeekClient {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl DeepSeekClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .expect("Failed to create HTTP client"),
            api_key: api_key.to_string(),
            base_url: "https://api.deepseek.com".to_string(),
        }
    }

    async fn get_raw(&self, path: &str) -> Result<(reqwest::StatusCode, String), ApiError> {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Accept", "application/json")
            .send()
            .await?;

        let status = response.status();
        let text = response.text().await.map_err(|e| {
            ApiError::ParseError(format!("读取响应失败: {e}"))
        })?;

        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(ApiError::InvalidApiKey);
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(ApiError::RateLimited);
        }

        Ok((status, text))
    }

    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
        let (status, text) = self.get_raw(path).await?;

        if !status.is_success() {
            return Err(ApiError::RequestError(format!("HTTP {}", status)));
        }

        serde_json::from_str(&text).map_err(|e| {
            ApiError::ParseError(format!(
                "JSON 解析失败: {e} | 原始数据: {}",
                &text[..text.len().min(200)]
            ))
        })
    }

    /// 查询余额（官方端点 GET /user/balance）
    pub async fn get_balance(&self) -> Result<BalanceResponse, ApiError> {
        self.get("/user/balance").await
    }

    /// 查询指定日期范围内的用量（非官方接口，多数账户返回 404）
    pub async fn fetch_usage(&self, days: u32) -> Result<Vec<UsageRecord>, ApiError> {
        let end = Local::now().date_naive();
        let start = end - chrono::Days::new(days.saturating_sub(1).max(0) as u64);
        let path = format!(
            "/v1/usage?start_date={}&end_date={}",
            start.format("%Y-%m-%d"),
            end.format("%Y-%m-%d")
        );

        let (status, text) = self.get_raw(&path).await?;

        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(ApiError::UsageEndpointUnavailable);
        }
        if !status.is_success() {
            return Err(ApiError::RequestError(format!("HTTP {status}")));
        }

        let response: UsageResponse = serde_json::from_str(&text).map_err(|e| {
            ApiError::ParseError(format!(
                "用量 JSON 解析失败: {e} | 原始数据: {}",
                &text[..text.len().min(200)]
            ))
        })?;

        Ok(response.data)
    }

    /// 调用 DeepSeek Chat 进行用量解读
    pub async fn chat_analyze(
        &self,
        user_prompt: &str,
        locale: &str,
    ) -> Result<String, ApiError> {
        let system_prompt = if locale == "en" {
            "You are a DeepSeek API usage analyst. Based on the JSON data, write a structured report in English using exactly this format (no intro, no markdown headers):\n\n【Overview】\nOne-line summary\n\n【Hit Rate Analysis】\n· Point one\n· Point two\n\n【Cost & Usage】\n· Point one\n· Point two\n\n【Recommendations】\n1. Suggestion one\n2. Suggestion two\n\nBe specific and readable. Mention cache hit rate."
        } else {
            "你是 DeepSeek API 用量分析助手。根据 JSON 数据输出结构化分析报告，严格按以下格式（不要前言、不要 markdown 标题）：\n\n【概要】\n一行总体评价\n\n【命中率分析】\n· 要点一\n· 要点二\n\n【消耗与成本】\n· 要点一\n· 要点二\n\n【优化建议】\n1. 建议一\n2. 建议二\n\n每条具体可读，必须提到缓存命中率。"
        };

        let url = format!("{}/chat/completions", self.base_url);
        let body = serde_json::json!({
            "model": "deepseek-chat",
            "messages": [
                {
                    "role": "system",
                    "content": system_prompt
                },
                { "role": "user", "content": user_prompt }
            ],
            "max_tokens": 520,
            "temperature": 0.5
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        let text = response.text().await.map_err(|e| {
            ApiError::ParseError(format!("读取响应失败: {e}"))
        })?;

        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(ApiError::InvalidApiKey);
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(ApiError::RateLimited);
        }
        if !status.is_success() {
            return Err(ApiError::RequestError(format!(
                "HTTP {status}: {}",
                &text[..text.len().min(200)]
            )));
        }

        let json: serde_json::Value = serde_json::from_str(&text).map_err(|e| {
            ApiError::ParseError(format!("Chat JSON 解析失败: {e}"))
        })?;

        json["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.trim().to_string())
            .ok_or_else(|| ApiError::ParseError("Chat 响应缺少 content".to_string()))
    }
}
