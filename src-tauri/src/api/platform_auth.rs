/// 注入用量页 WebView，拦截页面发出的 Authorization 头
pub const AUTH_HOOK_INIT_SCRIPT: &str = r#"
(function () {
  if (window.__DS_PLATFORM_TOKEN__) return;
  function remember(v) {
    if (!v) return;
    var t = String(v).replace(/^Bearer\s+/i, "").trim();
    if (t.length > 8) window.__DS_PLATFORM_TOKEN__ = t;
  }
  var xhrSet = XMLHttpRequest.prototype.setRequestHeader;
  XMLHttpRequest.prototype.setRequestHeader = function (name, value) {
    if (String(name).toLowerCase() === "authorization") remember(value);
    return xhrSet.apply(this, arguments);
  };
  if (window.fetch) {
    var origFetch = window.fetch;
    window.fetch = function (input, init) {
      try {
        if (init && init.headers) {
          if (init.headers instanceof Headers) {
            var h = init.headers.get("Authorization") || init.headers.get("authorization");
            remember(h);
          } else if (typeof init.headers === "object") {
            remember(
              init.headers.Authorization ||
                init.headers.authorization ||
                null
            );
          }
        }
      } catch (e) {}
      return origFetch.apply(this, arguments);
    };
  }
})();
"#;

/// 从 WebView 读取 platform 登录 Token（同步脚本）
pub const EXTRACT_TOKEN_JS: &str = r#"
(function () {
  function clean(raw) {
    if (!raw) return null;
    var v = String(raw).trim();
    if (!v || v === "null" || v === "undefined") return null;
    if (v.charAt(0) === "{" || v.charAt(0) === "[") {
      try {
        var o = JSON.parse(v);
        if (o && typeof o === "object") {
          return (
            (o.value != null && String(o.value)) ||
            o.token ||
            o.accessToken ||
            o.access_token ||
            o.userToken ||
            null
          );
        }
      } catch (e) {}
    }
    return v.replace(/^Bearer\s+/i, "");
  }

  function readKey(key) {
    try {
      var raw =
        localStorage.getItem(key) ||
        sessionStorage.getItem(key) ||
        null;
      var v = clean(raw);
      return v && v.length > 8 ? v : null;
    } catch (e) {
      return null;
    }
  }

  if (window.__DS_PLATFORM_TOKEN__) {
    return JSON.stringify({ ok: true, token: window.__DS_PLATFORM_TOKEN__, source: "hook" });
  }

  var keys = ["userToken", "settingsJwt", "token", "authToken", "accessToken"];
  for (var i = 0; i < keys.length; i++) {
    var t = readKey(keys[i]);
    if (t) return JSON.stringify({ ok: true, token: t, source: keys[i] });
  }

  var storageKeys = [];
  try {
    for (var j = 0; j < localStorage.length; j++) storageKeys.push(localStorage.key(j));
  } catch (e2) {}

  return JSON.stringify({
    ok: false,
    error: "未找到 userToken/settingsJwt，请先在窗口登录并等用量页加载完成",
    storageKeys: storageKeys.slice(0, 20),
  });
})()
"#;

pub fn token_from_cookie_value(name: &str, value: &str) -> Option<String> {
    let name = name.to_lowercase();
    let value = value.trim();
    if value.len() < 8 {
        return None;
    }
    if name.contains("token") || name.contains("jwt") || name == "authorization" {
        return Some(value.trim_start_matches("Bearer ").to_string());
    }
    if value.starts_with("eyJ") && value.len() > 20 {
        return Some(value.to_string());
    }
    None
}
