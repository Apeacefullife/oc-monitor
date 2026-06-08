use std::sync::mpsc;
use std::time::Duration;

use chrono::Datelike;
use serde::Deserialize;
use tauri::WebviewWindow;

use super::platform_auth::EXTRACT_TOKEN_JS;
use super::platform_usage::{self, NormalizedUsage};

#[derive(Debug, Deserialize)]
struct TokenExtractPayload {
    ok: bool,
    token: Option<String>,
    source: Option<String>,
    error: Option<String>,
    #[serde(rename = "storageKeys")]
    storage_keys: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct WebviewFetchPayload {
    ok: bool,
    status: Option<u16>,
    error: Option<String>,
    amount: Option<serde_json::Value>,
    #[serde(rename = "dailyAmount")]
    daily_amount: Option<serde_json::Value>,
    cost: Option<serde_json::Value>,
    #[serde(rename = "prevAmount")]
    prev_amount: Option<serde_json::Value>,
    token: Option<String>,
}

const DEFAULT_EVAL_TIMEOUT: Duration = Duration::from_secs(12);
const POLL_EVAL_TIMEOUT: Duration = Duration::from_secs(3);

fn eval_sync(window: &WebviewWindow, js: &str, timeout: Duration) -> Result<String, String> {
    let (tx, rx) = mpsc::channel::<String>();
    window
        .eval_with_callback(js, move |result| {
            let _ = tx.send(result);
        })
        .map_err(|e| format!("无法在用量页执行脚本: {e}"))?;
    rx.recv_timeout(timeout)
        .map_err(|_| "读取用量页数据超时，请确认已登录并停留在用量页".to_string())
}

fn parse_eval_string(raw: &str) -> Result<serde_json::Value, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed == "null" || trimmed == "undefined" {
        return Err("用量页脚本无返回".to_string());
    }
    let value: serde_json::Value = serde_json::from_str(trimmed)
        .map_err(|e| format!("用量页返回解析失败: {e}"))?;
    if value.is_string() {
        if let Ok(inner) = serde_json::from_str(value.as_str().unwrap_or("")) {
            return Ok(inner);
        }
    }
    Ok(value)
}

pub fn extract_token_in_webview(window: &WebviewWindow) -> Result<String, String> {
    extract_token_in_webview_with_timeout(window, DEFAULT_EVAL_TIMEOUT)
}

pub fn extract_token_in_webview_with_timeout(
    window: &WebviewWindow,
    timeout: Duration,
) -> Result<String, String> {
    let raw = eval_sync(window, EXTRACT_TOKEN_JS, timeout)?;
    let value = parse_eval_string(&raw)?;
    let payload: TokenExtractPayload =
        serde_json::from_value(value).map_err(|e| format!("Token 结果解析失败: {e}"))?;

    if payload.ok {
        return payload.token.ok_or_else(|| "Token 为空".to_string());
    }

    let hint = payload.error.unwrap_or_else(|| "未找到登录 Token".to_string());
    if let Some(keys) = payload.storage_keys.filter(|k| !k.is_empty()) {
        return Err(format!("{hint}（localStorage keys: {}）", keys.join(", ")));
    }
    Err(hint)
}

fn build_fetch_js(token: &str) -> String {
    let token_json = serde_json::to_string(token).unwrap_or_else(|_| "\"\"".to_string());
    format!(
        r#"
(function () {{
  var token = {token_json};

  function unwrapPlatform(data) {{
    if (!data || typeof data !== "object") return {{ ok: false, error: "空响应" }};
    if (typeof data.code === "number" && data.code !== 0) {{
      return {{
        ok: false,
        code: data.code,
        error: data.msg || "平台 API code " + data.code,
      }};
    }}
    if (Object.prototype.hasOwnProperty.call(data, "data")) {{
      if (data.data === null || data.data === undefined) {{
        return {{ ok: false, error: data.msg || "平台返回空 data" }};
      }}
      return {{ ok: true, data: data.data }};
    }}
    return {{ ok: true, data: data }};
  }}

  function syncGet(url) {{
    var xhr = new XMLHttpRequest();
    xhr.open("GET", url, false);
    xhr.withCredentials = true;
    xhr.setRequestHeader("Accept", "application/json");
    xhr.setRequestHeader("Authorization", "Bearer " + token);
    try {{
      xhr.send(null);
    }} catch (e) {{
      return {{ ok: false, error: String(e) }};
    }}
    if (xhr.status < 200 || xhr.status >= 300) {{
      return {{ ok: false, status: xhr.status, error: "HTTP " + xhr.status }};
    }}
    try {{
      var parsed = JSON.parse(xhr.responseText);
      var unwrapped = unwrapPlatform(parsed);
      if (!unwrapped.ok) {{
        return {{
          ok: false,
          status: unwrapped.code || null,
          error: unwrapped.error || "平台 API 错误",
        }};
      }}
      return {{ ok: true, data: unwrapped.data }};
    }} catch (e) {{
      return {{ ok: false, error: "JSON: " + e }};
    }}
  }}

  var now = new Date();
  var y = now.getFullYear();
  var m = now.getMonth() + 1;
  var base = "https://platform.deepseek.com/api/v0/usage/";
  var amountRes = syncGet(base + "amount?year=" + y + "&month=" + m);
  if (!amountRes.ok) {{
    return {{
      ok: false,
      status: amountRes.status || null,
      error: amountRes.error || "amount 请求失败",
      token: token,
    }};
  }}
  var dailyRes = syncGet(base + "amount?year=" + y + "&month=" + m + "&group_by=day");
  var dailyAmount = dailyRes.ok ? dailyRes.data : null;
  var costRes = syncGet(base + "cost?year=" + y + "&month=" + m);
  var cost = costRes.ok ? costRes.data : null;
  var prevAmount = null;
  if (now.getDate() <= 7) {{
    var py = y;
    var pm = m - 1;
    if (pm < 1) {{
      pm = 12;
      py = y - 1;
    }}
    var prevRes = syncGet(base + "amount?year=" + py + "&month=" + pm + "&group_by=day");
    if (prevRes.ok) prevAmount = prevRes.data;
  }}
  return {{
    ok: true,
    amount: amountRes.data,
    dailyAmount: dailyAmount,
    cost: cost,
    prevAmount: prevAmount,
    token: token,
  }};
}})()
"#
    )
}

pub fn fetch_usage_in_webview(
    window: &WebviewWindow,
    cookie_token: Option<&str>,
) -> Result<(NormalizedUsage, Option<String>), String> {
    fetch_usage_in_webview_with_timeout(window, cookie_token, DEFAULT_EVAL_TIMEOUT)
}

pub fn fetch_usage_in_webview_with_timeout(
    window: &WebviewWindow,
    cookie_token: Option<&str>,
    timeout: Duration,
) -> Result<(NormalizedUsage, Option<String>), String> {
    let token = match extract_token_in_webview_with_timeout(window, timeout) {
        Ok(t) => t,
        Err(js_err) => cookie_token
            .map(str::to_string)
            .ok_or(js_err)?,
    };

    let js = build_fetch_js(&token);
    let raw = eval_sync(window, &js, timeout)?;
    parse_webview_fetch_result(&raw)
}

pub fn poll_eval_timeout() -> Duration {
    POLL_EVAL_TIMEOUT
}

fn parse_webview_fetch_result(raw: &str) -> Result<(NormalizedUsage, Option<String>), String> {
    let value = parse_eval_string(raw)?;
    let payload: WebviewFetchPayload =
        serde_json::from_value(value).map_err(|e| format!("用量页数据结构异常: {e}"))?;

    if !payload.ok {
        let err = payload.error.unwrap_or_default();
        if err.contains("Missing Token") || payload.status == Some(40002) {
            return Err(
                "登录 Token 无效或已过期：请在用量页窗口重新登录，看到用量数据后再同步"
                    .to_string(),
            );
        }
        if let Some(status) = payload.status {
            return Err(format!("用量页接口错误 {status}：{err}"));
        }
        return Err(if err.is_empty() {
            "用量页抓取失败".to_string()
        } else {
            err
        });
    }

    let amount = payload
        .amount
        .ok_or("用量页未返回 amount 数据".to_string())?;

    let usage = platform_usage::normalize(
        &amount,
        payload.cost.as_ref(),
        payload.daily_amount.as_ref(),
        payload.prev_amount.as_ref(),
    )
    .ok_or_else(|| {
        let preview = serde_json::to_string(&amount)
            .unwrap_or_default()
            .chars()
            .take(200)
            .collect::<String>();
        let now = chrono::Local::now();
        format!(
            "用量解析为空（{}-{:02}）。原始片段: {preview}",
            now.year(),
            now.month()
        )
    })?;

    Ok((usage, payload.token))
}
