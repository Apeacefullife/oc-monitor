use std::fmt;

/// API 错误类型
#[derive(Debug)]
pub enum ApiError {
    /// HTTP 请求错误
    RequestError(String),
    /// API 返回错误
    ApiReturnError { code: i32, message: String },
    /// API Key 无效
    InvalidApiKey,
    /// 网络超时
    Timeout,
    /// JSON 解析错误
    ParseError(String),
    /// 速率限制
    RateLimited,
    /// 官方用量接口未开放（HTTP 404）
    UsageEndpointUnavailable,
}

impl ApiError {
    pub fn status_code(&self) -> Option<u16> {
        match self {
            ApiError::UsageEndpointUnavailable => Some(404),
            ApiError::InvalidApiKey => Some(401),
            ApiError::RateLimited => Some(429),
            ApiError::RequestError(msg) => msg
                .strip_prefix("HTTP ")
                .and_then(|s| s.parse().ok()),
            _ => None,
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::RequestError(msg) => write!(f, "网络请求失败: {}", msg),
            ApiError::ApiReturnError { code, message } => {
                write!(f, "API 错误 ({}): {}", code, message)
            }
            ApiError::InvalidApiKey => write!(f, "API Key 无效，请检查设置"),
            ApiError::Timeout => write!(f, "请求超时，请检查网络连接"),
            ApiError::ParseError(msg) => write!(f, "数据解析失败: {}", msg),
            ApiError::RateLimited => write!(f, "请求过于频繁，请稍后再试"),
            ApiError::UsageEndpointUnavailable => {
                write!(f, "DeepSeek 当前未公开用量查询接口，已仅显示余额")
            }
        }
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            ApiError::Timeout
        } else if err.is_status() {
            match err.status().unwrap().as_u16() {
                401 => ApiError::InvalidApiKey,
                429 => ApiError::RateLimited,
                code => ApiError::RequestError(format!("HTTP {}", code)),
            }
        } else {
            ApiError::RequestError(err.to_string())
        }
    }
}
