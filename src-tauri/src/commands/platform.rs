use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager, Url, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

const SILENT_CAPTURE_MAX_ATTEMPTS: u32 = 6;
const SILENT_CAPTURE_INTERVAL_SECS: u64 = 2;

static PLATFORM_FETCH_ACTIVE: AtomicBool = AtomicBool::new(false);
static LOGIN_POLL_ACTIVE: AtomicBool = AtomicBool::new(false);

const PLATFORM_LOGIN_LABEL: &str = "platform-login";

use crate::api::platform_auth::{self, AUTH_HOOK_INIT_SCRIPT};
use crate::api::platform_usage::{self, NormalizedUsage};
use crate::api::platform_webview;

use crate::store::crypto;

const PLATFORM_COOKIE_KEY: &str = "platform_cookie";
const PLATFORM_TOKEN_KEY: &str = "platform_token";
const SILENT_PLATFORM_LABEL: &str = "platform-silent";

fn load_platform_cookie(app: &AppHandle) -> Result<Option<String>, String> {
    let store = crate::store::get_store(app);
    match store.get(PLATFORM_COOKIE_KEY) {
        Some(serde_json::Value::String(encrypted)) => {
            let decrypted = crypto::decrypt(&encrypted).map_err(|e| e.to_string())?;
            Ok(Some(decrypted))
        }
        _ => Ok(None),
    }
}

fn load_platform_token(app: &AppHandle) -> Result<Option<String>, String> {
    let store = crate::store::get_store(app);
    match store.get(PLATFORM_TOKEN_KEY) {
        Some(serde_json::Value::String(encrypted)) => {
            let decrypted = crypto::decrypt(&encrypted).map_err(|e| e.to_string())?;
            Ok(Some(decrypted))
        }
        _ => Ok(None),
    }
}

fn save_platform_cookie(app: &AppHandle, cookie: &str) -> Result<(), String> {
    let store = crate::store::get_store(app);
    let encrypted = crypto::encrypt(cookie).map_err(|e| e.to_string())?;
    store.set(PLATFORM_COOKIE_KEY, serde_json::Value::String(encrypted));
    store
        .save()
        .map_err(|e| format!("保存登录状态失败: {e}"))
}

fn save_platform_token(app: &AppHandle, token: &str) -> Result<(), String> {
    let store = crate::store::get_store(app);
    let encrypted = crypto::encrypt(token).map_err(|e| e.to_string())?;
    store.set(PLATFORM_TOKEN_KEY, serde_json::Value::String(encrypted));
    store
        .save()
        .map_err(|e| format!("保存登录 Token 失败: {e}"))
}

fn collect_cookies_from_window(
    window: &WebviewWindow,
) -> Result<Vec<tauri::webview::Cookie<'_>>, String> {
    let urls = [
        "https://platform.deepseek.com",
        "https://platform.deepseek.com/usage",
        "https://deepseek.com",
    ];

    let mut merged = Vec::new();
    for url in urls {
        if let Ok(parsed) = Url::parse(url) {
            if let Ok(list) = window.cookies_for_url(parsed) {
                for cookie in list {
                    if !merged.iter().any(|c: &tauri::webview::Cookie| {
                        c.name() == cookie.name() && c.domain() == cookie.domain()
                    }) {
                        merged.push(cookie);
                    }
                }
            }
        }
    }

    Ok(merged)
}

fn cookies_to_header(cookies: &[tauri::webview::Cookie<'_>]) -> String {
    cookies
        .iter()
        .map(|c| format!("{}={}", c.name(), c.value()))
        .collect::<Vec<_>>()
        .join("; ")
}

fn extract_token_from_cookies(cookies: &[tauri::webview::Cookie<'_>]) -> Option<String> {
    cookies
        .iter()
        .find_map(|c| platform_auth::token_from_cookie_value(c.name(), c.value()))
}

fn load_cached_platform_usage(app: &AppHandle) -> Option<NormalizedUsage> {
    let store = crate::store::get_store(app);
    store
        .get("cached_platform_usage")
        .and_then(|v| serde_json::from_value(v).ok())
}

fn persist_platform_usage(app: &AppHandle, usage: &NormalizedUsage) {
    let store = crate::store::get_store(app);
    if let Ok(value) = serde_json::to_value(usage) {
        store.set("cached_platform_usage", value);
        let _ = store.save();
    }
}

pub fn has_platform_credentials(app: &AppHandle) -> bool {
    load_platform_cookie(app)
        .ok()
        .flatten()
        .is_some()
        || load_platform_token(app)
            .ok()
            .flatten()
            .is_some()
}

pub fn has_saved_platform_auth(app: &AppHandle) -> bool {
    has_platform_credentials(app) || load_cached_platform_usage(app).is_some()
}

fn is_platform_auth_failure(err: &str) -> bool {
    err.contains("40002")
        || err.contains("未登录")
        || err.contains("Token 已过期")
        || err.contains("Missing Token")
        || err.contains("缺少平台登录凭证")
}

pub fn clear_platform_session(app: &AppHandle) {
    let store = crate::store::get_store(app);
    store.delete(PLATFORM_COOKIE_KEY);
    store.delete(PLATFORM_TOKEN_KEY);
    let _ = store.save();
}

pub async fn fetch_platform_usage(app: &AppHandle) -> Result<NormalizedUsage, String> {
    let cookie = load_platform_cookie(app)?;
    let token = load_platform_token(app)?;
    platform_usage::fetch_usage(cookie.as_deref(), token.as_deref()).await
}

fn persist_session_from_window(
    app: &AppHandle,
    window: &WebviewWindow,
    usage: &NormalizedUsage,
    token: Option<String>,
) {
    persist_platform_usage(app, usage);
    if let Some(token) = token.as_deref() {
        let _ = save_platform_token(app, token);
    }
    if let Ok(cookies) = collect_cookies_from_window(window) {
        if !cookies.is_empty() {
            let _ = save_platform_cookie(app, &cookies_to_header(&cookies));
        }
    }
}

fn try_capture_from_window_with_timeout(
    app: &AppHandle,
    window: &WebviewWindow,
    eval_timeout: Duration,
) -> Option<NormalizedUsage> {
    let _ = window.eval(AUTH_HOOK_INIT_SCRIPT);
    let cookies = collect_cookies_from_window(window).unwrap_or_default();
    let cookie_token = extract_token_from_cookies(&cookies);
    let (usage, token) = platform_webview::fetch_usage_in_webview_with_timeout(
        window,
        cookie_token.as_deref(),
        eval_timeout,
    )
    .ok()?;
    persist_session_from_window(app, window, &usage, token);
    Some(usage)
}

fn try_capture_from_window(app: &AppHandle, window: &WebviewWindow) -> Option<NormalizedUsage> {
    try_capture_from_window_with_timeout(
        app,
        window,
        std::time::Duration::from_secs(12),
    )
}

async fn try_capture_from_window_async(
    app: AppHandle,
    label: String,
    eval_timeout: Duration,
) -> Option<NormalizedUsage> {
    tokio::task::spawn_blocking(move || {
        let window = app.get_webview_window(&label)?;
        try_capture_from_window_with_timeout(&app, &window, eval_timeout)
    })
    .await
    .ok()
    .flatten()
}

async fn ensure_silent_webview(app: &AppHandle) -> Option<WebviewWindow> {
    if let Some(window) = app.get_webview_window(SILENT_PLATFORM_LABEL) {
        let _ = window.eval(AUTH_HOOK_INIT_SCRIPT);
        return Some(window);
    }

    for label in ["platform-login"] {
        if let Some(window) = app.get_webview_window(label) {
            let _ = window.close();
        }
    }

    let url: Url = "https://platform.deepseek.com/usage"
        .parse()
        .map_err(|e| format!("URL 无效: {e}"))
        .ok()?;

    WebviewWindowBuilder::new(app, SILENT_PLATFORM_LABEL, WebviewUrl::External(url))
        .title("DS-Monitor")
        .visible(false)
        .skip_taskbar(true)
        .decorations(false)
        .resizable(false)
        .inner_size(960.0, 720.0)
        .initialization_script(AUTH_HOOK_INIT_SCRIPT)
        .build()
        .ok()
}

async fn capture_from_silent_webview(app: &AppHandle) -> Option<NormalizedUsage> {
    if PLATFORM_FETCH_ACTIVE.swap(true, Ordering::SeqCst) {
        return load_cached_platform_usage(app);
    }

    let result = capture_from_silent_webview_inner(app).await;
    PLATFORM_FETCH_ACTIVE.store(false, Ordering::SeqCst);
    result
}

async fn capture_from_silent_webview_inner(app: &AppHandle) -> Option<NormalizedUsage> {
    let window = ensure_silent_webview(app).await?;
    let label = window.label().to_string();
    let app_handle = app.clone();
    let poll_timeout = platform_webview::poll_eval_timeout();

    for attempt in 0..SILENT_CAPTURE_MAX_ATTEMPTS {
        if attempt > 0 {
            tokio::time::sleep(Duration::from_secs(SILENT_CAPTURE_INTERVAL_SECS)).await;
        }
        if let Some(usage) = try_capture_from_window_async(
            app_handle.clone(),
            label.clone(),
            poll_timeout,
        )
        .await
        {
            return Some(usage);
        }
    }

    None
}

/// 静默拉取平台用量：Token/Cookie → 隐藏 WebView → 本地缓存
pub async fn silent_fetch_platform_usage(app: &AppHandle) -> Option<NormalizedUsage> {
    let had_credentials = has_platform_credentials(app);

    if had_credentials {
        match fetch_platform_usage(app).await {
            Ok(usage) => {
                persist_platform_usage(app, &usage);
                return Some(usage);
            }
            Err(ref e) if is_platform_auth_failure(e) => {
                if let Some(usage) = capture_from_silent_webview(app).await {
                    return Some(usage);
                }
                clear_platform_session(app);
                let _ = app.emit("platform-session-expired", ());
                return load_cached_platform_usage(app);
            }
            Err(_) => {
                return load_cached_platform_usage(app);
            }
        }
    }

    load_cached_platform_usage(app)
}

async fn poll_platform_login_window(app: AppHandle) {
    if LOGIN_POLL_ACTIVE.swap(true, Ordering::SeqCst) {
        return;
    }

    let mut success = false;
    let poll_timeout = platform_webview::poll_eval_timeout();

    for _ in 0..90 {
        tokio::time::sleep(Duration::from_secs(2)).await;

        let Some(window) = app.get_webview_window(PLATFORM_LOGIN_LABEL) else {
            break;
        };
        let label = window.label().to_string();

        if let Some(_usage) =
            try_capture_from_window_async(app.clone(), label, poll_timeout).await
        {
            success = true;
            if let Some(window) = app.get_webview_window(PLATFORM_LOGIN_LABEL) {
                let _ = window.close();
            }
            let handle = app.clone();
            tauri::async_runtime::spawn(async move {
                let _ = crate::commands::refresh::silent_refresh(handle).await;
            });
            let _ = app.emit("platform-login-done", ());
            break;
        }
    }

    if !success {
        let _ = app.emit("platform-login-cancelled", ());
    }

    LOGIN_POLL_ACTIVE.store(false, Ordering::SeqCst);
}

/// 是否已有平台登录凭证（Token / Cookie，不含仅本地缓存）
#[tauri::command]
pub async fn has_platform_session(app_handle: AppHandle) -> Result<bool, String> {
    Ok(has_platform_credentials(&app_handle))
}

/// 打开 DeepSeek 平台登录窗口，登录成功后自动同步用量
#[tauri::command]
pub async fn open_platform_login(app_handle: AppHandle) -> Result<bool, String> {
    if let Some(window) = app_handle.get_webview_window(SILENT_PLATFORM_LABEL) {
        let _ = window.close();
    }

    if let Some(window) = app_handle.get_webview_window(PLATFORM_LOGIN_LABEL) {
        let _ = window.show();
        let _ = window.set_focus();
        let _ = app_handle.emit("platform-login-opening", ());
        let handle = app_handle.clone();
        tauri::async_runtime::spawn(async move {
            poll_platform_login_window(handle).await;
        });
        return Ok(true);
    }

    let url: Url = "https://platform.deepseek.com/sign_in"
        .parse()
        .map_err(|e| format!("URL 无效: {e}"))?;

    WebviewWindowBuilder::new(&app_handle, PLATFORM_LOGIN_LABEL, WebviewUrl::External(url))
        .title("DeepSeek 平台登录 · DS-Monitor")
        .visible(true)
        .center()
        .decorations(true)
        .resizable(true)
        .inner_size(480.0, 680.0)
        .initialization_script(AUTH_HOOK_INIT_SCRIPT)
        .build()
        .map_err(|e| format!("无法打开登录窗口: {e}"))?;

    let _ = app_handle.emit("platform-login-opening", ());

    let handle = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        poll_platform_login_window(handle).await;
    });

    Ok(true)
}
