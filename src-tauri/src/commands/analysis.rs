use tauri::AppHandle;

use crate::api::client::DeepSeekClient;
use crate::commands::settings;

#[derive(Debug, serde::Serialize)]
pub struct AiAnalysisResponse {
    pub success: bool,
    pub content: Option<String>,
    pub error: Option<String>,
}

/// 调用 DeepSeek Chat 解读用量数据（含缓存命中率）
#[tauri::command]
pub async fn analyze_usage_ai(
    app_handle: AppHandle,
    payload: String,
    locale: Option<String>,
) -> Result<AiAnalysisResponse, String> {
    let lang = locale.as_deref().unwrap_or("zh");
    let api_key_missing = if lang == "en" {
        "Please set your API Key first"
    } else {
        "请先设置 API Key"
    };

    let Some(api_key) = settings::load_api_key(&app_handle)? else {
        return Ok(AiAnalysisResponse {
            success: false,
            content: None,
            error: Some(api_key_missing.to_string()),
        });
    };

    let client = DeepSeekClient::new(&api_key);

    match client.chat_analyze(&payload, lang).await {
        Ok(content) => Ok(AiAnalysisResponse {
            success: true,
            content: Some(content),
            error: None,
        }),
        Err(e) => Ok(AiAnalysisResponse {
            success: false,
            content: None,
            error: Some(e.to_string()),
        }),
    }
}
