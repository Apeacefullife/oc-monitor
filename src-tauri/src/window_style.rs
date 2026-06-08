//! Windows 窗口外观：无边框 + 抑制原生 NC 绘制 + SWCA 磨玻璃
//!
//! 原生边框闪一下的根因：
//! 1. 反复 `set_decorations(false)` 会触发 WebView2 重绘非客户区
//! 2. DWM / WebView2 在 resize / focus 时会尝试绘制默认边框
//! 解决：WndProc 拦截 WM_NCPAINT、DwmExtendFrameIntoClientArea、仅初始化一次 chrome

use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicU32, AtomicU64, Ordering};
use std::sync::OnceLock;

use tauri::{Emitter, Manager, WebviewWindow, window::Color};

use crate::window_interaction;

/// SWCA 磨玻璃着色（ARGB，alpha 越低越透）
const ACRYLIC_TINT: Color = Color(13, 17, 23, 100);

const MAINTAIN_COOLDOWN_MS: u64 = 400;
const WATCHDOG_INTERVAL_SECS: u64 = 8;

static LAST_MAINTAIN_MS: AtomicU64 = AtomicU64::new(0);
static RESIZE_GENERATION: AtomicU64 = AtomicU64::new(0);
static LAYOUT_TRANSITION: AtomicBool = AtomicBool::new(false);
static CHROME_INITIALIZED: AtomicBool = AtomicBool::new(false);
static WNDPROC_INSTALLED: AtomicBool = AtomicBool::new(false);
static ORIGINAL_WNDPROC: AtomicIsize = AtomicIsize::new(0);
static WINDOW_SCALE_CENTI: AtomicU32 = AtomicU32::new(100);

#[cfg(target_os = "windows")]
static BLANK_CURSOR: OnceLock<isize> = OnceLock::new();

#[cfg(target_os = "windows")]
static CURSOR_EVENT_TARGET: OnceLock<(tauri::AppHandle, String)> = OnceLock::new();

#[cfg(target_os = "windows")]
static MOUSE_LEAVE_TRACKING: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "windows")]
static CURSOR_WAS_INSIDE: AtomicBool = AtomicBool::new(false);

pub fn start_cursor_bounds_watchdog(app: tauri::AppHandle, label: impl Into<String>) {
    let label = label.into();
    #[cfg(target_os = "windows")]
    {
        let _ = CURSOR_EVENT_TARGET.set((app.clone(), label.clone()));
        tauri::async_runtime::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(16)).await;
                let Some((target_app, target_label)) = CURSOR_EVENT_TARGET.get() else {
                    continue;
                };
                let Some(window) = target_app.get_webview_window(target_label) else {
                    continue;
                };
                if !window.is_visible().unwrap_or(false) {
                    CURSOR_WAS_INSIDE.store(false, Ordering::Relaxed);
                    continue;
                }
                let inside = cursor_inside_window(&window);
                let was_inside = CURSOR_WAS_INSIDE.swap(inside, Ordering::Relaxed);
                if was_inside && !inside {
                    let _ = window.emit("window-mouse-leave", ());
                }
            }
        });
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (app, label);
    }
}

type WndProcFn = unsafe extern "system" fn(
    windows::Win32::Foundation::HWND,
    u32,
    windows::Win32::Foundation::WPARAM,
    windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT;

pub fn set_layout_transitioning(active: bool) {
    LAYOUT_TRANSITION.store(active, Ordering::Relaxed);
}

pub fn update_scale(window: &WebviewWindow) {
    if let Ok(scale) = window.scale_factor() {
        WINDOW_SCALE_CENTI.store((scale * 100.0).round() as u32, Ordering::Relaxed);
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn should_skip_maintain() -> bool {
    let now = now_ms();
    let last = LAST_MAINTAIN_MS.load(Ordering::Relaxed);
    if now.saturating_sub(last) < MAINTAIN_COOLDOWN_MS {
        return true;
    }
    LAST_MAINTAIN_MS.store(now, Ordering::Relaxed);
    false
}

/// 启动 / 托盘恢复：完整初始化或轻量维护
pub fn apply_force(window: &WebviewWindow) {
    if !CHROME_INITIALIZED.load(Ordering::Acquire) {
        apply_chrome_init(window);
        CHROME_INITIALIZED.store(true, Ordering::Release);
    } else {
        maintain_chrome(window);
    }
}

/// 轻量刷新（resize / watchdog）—— 不触碰 decorations
pub fn apply_maintain(window: &WebviewWindow) {
    if should_skip_maintain() {
        return;
    }
    maintain_chrome(window);
}

/// 显示窗口前：确保 chrome 就绪（不触发 decorations 重设）
pub fn prepare_show(window: &WebviewWindow) {
    if !CHROME_INITIALIZED.load(Ordering::Acquire) {
        apply_chrome_init(window);
        CHROME_INITIALIZED.store(true, Ordering::Release);
    } else {
        maintain_chrome(window);
    }
}

fn apply_chrome_init(window: &WebviewWindow) {
    let _ = window.set_decorations(false);
    let _ = window.set_maximizable(false);
    let _ = window.set_cursor_visible(false);
    let _ = window.set_background_color(Some(Color(0, 0, 0, 0)));
    update_scale(window);

    #[cfg(target_os = "windows")]
    if let Ok(hwnd) = window.hwnd() {
        let raw = hwnd.0 as isize;
        let _ = ensure_wndproc_hook(window);
        strip_native_frame(raw);
        apply_extend_frame(raw);
        apply_dwm(raw);
        apply_swca_acrylic(raw);
    }
}

fn maintain_chrome(window: &WebviewWindow) {
    let _ = window.set_background_color(Some(Color(0, 0, 0, 0)));
    let _ = window.set_cursor_visible(false);
    update_scale(window);

    #[cfg(target_os = "windows")]
    if let Ok(hwnd) = window.hwnd() {
        let raw = hwnd.0 as isize;
        apply_dwm(raw);
        apply_swca_acrylic(raw);
    }
}

#[cfg(target_os = "windows")]
pub fn ensure_wndproc_hook(window: &WebviewWindow) -> Result<(), String> {
    if WNDPROC_INSTALLED.load(Ordering::Acquire) {
        return Ok(());
    }

    let hwnd = window.hwnd().map_err(|e| e.to_string())?;

    unsafe {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::{GetWindowLongPtrW, SetWindowLongPtrW, GWLP_WNDPROC};

        let hwnd = HWND(hwnd.0 as *mut _);
        let prev = GetWindowLongPtrW(hwnd, GWLP_WNDPROC);
        if prev == 0 {
            return Err("GetWindowLongPtrW failed".into());
        }

        let custom = chrome_wndproc as *const () as isize;
        let replaced = SetWindowLongPtrW(hwnd, GWLP_WNDPROC, custom);
        if replaced == 0 {
            return Err("SetWindowLongPtrW failed".into());
        }

        ORIGINAL_WNDPROC.store(replaced, Ordering::Relaxed);
        WNDPROC_INSTALLED.store(true, Ordering::Release);
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn ensure_wndproc_hook(_window: &WebviewWindow) -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "windows")]
fn strip_native_frame(hwnd: isize) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE, GWL_STYLE,
        SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, WS_BORDER,
        WS_CAPTION, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_SYSMENU, WS_THICKFRAME, WS_EX_CLIENTEDGE,
        WS_EX_WINDOWEDGE,
    };

    unsafe {
        let hwnd = HWND(hwnd as *mut _);
        let style = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;

        let style = style
            & !(WS_CAPTION | WS_THICKFRAME | WS_BORDER | WS_SYSMENU | WS_MINIMIZEBOX | WS_MAXIMIZEBOX).0;
        let ex_style = ex_style & !(WS_EX_CLIENTEDGE | WS_EX_WINDOWEDGE).0;

        let _ = SetWindowLongPtrW(hwnd, GWL_STYLE, style as isize);
        let _ = SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style as isize);
        let _ = SetWindowPos(
            hwnd,
            None,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        );
    }
}

#[cfg(target_os = "windows")]
fn apply_extend_frame(hwnd: isize) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Dwm::DwmExtendFrameIntoClientArea;
    use windows::Win32::UI::Controls::MARGINS;

    let margins = MARGINS {
        cxLeftWidth: -1,
        cxRightWidth: -1,
        cyTopHeight: -1,
        cyBottomHeight: -1,
    };

    unsafe {
        let _ = DwmExtendFrameIntoClientArea(HWND(hwnd as *mut _), &margins);
    }
}

#[cfg(target_os = "windows")]
fn apply_dwm(hwnd: isize) {
    use windows::Win32::Foundation::{HWND, FALSE};
    use windows::Win32::Graphics::Dwm::{
        DwmSetWindowAttribute, DWMWA_ALLOW_NCPAINT, DWMWA_BORDER_COLOR, DWMWA_CAPTION_COLOR,
        DWMWA_NCRENDERING_POLICY, DWMWA_TEXT_COLOR, DWMWA_TRANSITIONS_FORCEDISABLED,
        DWMWA_USE_IMMERSIVE_DARK_MODE, DWMWA_WINDOW_CORNER_PREFERENCE, DWMNCRP_DISABLED,
        DWMWCP_ROUND,
    };
    use windows::Win32::UI::WindowsAndMessaging::{GetClassLongPtrW, SetClassLongPtrW, GCL_STYLE};

    const CS_DROPSHADOW: isize = 0x0002_0000;
    const DWM_COLOR_NONE: u32 = 0xFFFF_FFFE;

    let hwnd = HWND(hwnd as *mut _);
    let immersive_dark: u32 = 1;
    let disable_transitions: u32 = 1;
    let corner = DWMWCP_ROUND.0 as u32;
    let nc_disabled = DWMNCRP_DISABLED.0 as u32;
    let border_none = DWM_COLOR_NONE;
    let caption_none = DWM_COLOR_NONE;
    let text_none = DWM_COLOR_NONE;
    let disallow_nc_paint = FALSE;

    unsafe {
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &immersive_dark as *const _ as *const _,
            std::mem::size_of::<u32>() as u32,
        );
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_TRANSITIONS_FORCEDISABLED,
            &disable_transitions as *const _ as *const _,
            std::mem::size_of::<u32>() as u32,
        );
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_NCRENDERING_POLICY,
            &nc_disabled as *const _ as *const _,
            std::mem::size_of::<u32>() as u32,
        );
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_ALLOW_NCPAINT,
            &disallow_nc_paint as *const _ as *const _,
            std::mem::size_of::<i32>() as u32,
        );
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &corner as *const _ as *const _,
            std::mem::size_of::<u32>() as u32,
        );
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_BORDER_COLOR,
            &border_none as *const _ as *const _,
            std::mem::size_of::<u32>() as u32,
        );
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_CAPTION_COLOR,
            &caption_none as *const _ as *const _,
            std::mem::size_of::<u32>() as u32,
        );
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_TEXT_COLOR,
            &text_none as *const _ as *const _,
            std::mem::size_of::<u32>() as u32,
        );

        let class_style = GetClassLongPtrW(hwnd, GCL_STYLE) as isize;
        if class_style != 0 {
            let _ = SetClassLongPtrW(hwnd, GCL_STYLE, class_style & !CS_DROPSHADOW);
        }
    }
}

#[cfg(target_os = "windows")]
fn apply_swca_acrylic(hwnd: isize) {
    use std::ffi::c_void;

    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};

    #[repr(C)]
    struct AccentPolicy {
        accent_state: u32,
        accent_flags: u32,
        gradient_color: u32,
        animation_id: u32,
    }

    #[repr(C)]
    struct WindowCompositionAttribData {
        attrib: u32,
        data: *mut c_void,
        size: usize,
    }

    type SetWindowCompositionAttributeFn =
        unsafe extern "system" fn(HWND, *mut WindowCompositionAttribData) -> i32;

    const ACCENT_ENABLE_ACRYLICBLURBEHIND: u32 = 4;
    const WCA_ACCENT_POLICY: u32 = 19;

    let alpha = ACRYLIC_TINT.3.max(1);
    let gradient_color = (ACRYLIC_TINT.0 as u32)
        | ((ACRYLIC_TINT.1 as u32) << 8)
        | ((ACRYLIC_TINT.2 as u32) << 16)
        | ((alpha as u32) << 24);

    unsafe {
        let user32 = LoadLibraryA(windows::core::s!("user32.dll")).ok();
        let Some(user32) = user32 else {
            return;
        };
        let proc = GetProcAddress(user32, windows::core::s!("SetWindowCompositionAttribute"));
        let Some(proc) = proc else {
            return;
        };
        let set_window_composition_attribute: SetWindowCompositionAttributeFn =
            std::mem::transmute(proc);

        let mut policy = AccentPolicy {
            accent_state: ACCENT_ENABLE_ACRYLICBLURBEHIND,
            accent_flags: 0,
            gradient_color,
            animation_id: 0,
        };
        let mut data = WindowCompositionAttribData {
            attrib: WCA_ACCENT_POLICY,
            data: &mut policy as *mut _ as *mut c_void,
            size: std::mem::size_of::<AccentPolicy>(),
        };
        let _ = set_window_composition_attribute(HWND(hwnd as *mut _), &mut data);
    }
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn chrome_wndproc(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::Foundation::{LRESULT, POINT, RECT};
    use windows::Win32::Graphics::Gdi::ScreenToClient;
    use windows::Win32::UI::WindowsAndMessaging::{
        CallWindowProcW, GetClientRect, HTCLIENT, HTTRANSPARENT, SC_MAXIMIZE, SC_RESTORE,
        WM_ERASEBKGND, WM_MOUSEMOVE, WM_NCACTIVATE, WM_NCHITTEST, WM_NCLBUTTONDBLCLK,
        WM_NCMOUSEMOVE, WM_NCPAINT, WM_SETCURSOR, WM_SYSCOMMAND,
    };
    use windows::Win32::UI::Controls::WM_MOUSELEAVE;
    use windows::Win32::UI::WindowsAndMessaging::WM_NCMOUSELEAVE;

    match msg {
        WM_MOUSEMOVE | WM_NCMOUSEMOVE => {
            track_mouse_leave(hwnd);
            apply_blank_system_cursor();
        }
        WM_MOUSELEAVE | WM_NCMOUSELEAVE => {
            MOUSE_LEAVE_TRACKING.store(false, Ordering::Relaxed);
            emit_window_mouse_leave();
        }
        WM_SETCURSOR => {
            let cursor_hwnd = windows::Win32::Foundation::HWND(wparam.0 as *mut _);
            if cursor_belongs_to_window(hwnd, cursor_hwnd) {
                apply_blank_system_cursor();
                return LRESULT(1);
            }
        }
        WM_NCPAINT | WM_ERASEBKGND => return LRESULT(1),
        WM_NCACTIVATE => return LRESULT(1),
        WM_NCLBUTTONDBLCLK => return LRESULT(0),
        WM_SYSCOMMAND => {
            let cmd = (wparam.0 & 0xFFF0) as u32;
            if cmd == SC_MAXIMIZE || cmd == SC_RESTORE {
                return LRESULT(0);
            }
        }
        WM_NCHITTEST if window_interaction::is_locked() => {
            let scale = WINDOW_SCALE_CENTI.load(Ordering::Relaxed) as f64 / 100.0;
            let titlebar_h =
                (window_interaction::titlebar_height_logical() * scale).round() as i32;
            let actions_w =
                (window_interaction::titlebar_actions_width_logical() * scale).round() as i32;

            let raw = lparam.0 as isize;
            let screen_x = (raw & 0xFFFF) as i16 as i32;
            let screen_y = ((raw >> 16) & 0xFFFF) as i16 as i32;
            let mut pt = POINT {
                x: screen_x,
                y: screen_y,
            };
            let _ = ScreenToClient(hwnd, &mut pt);

            let mut rect = RECT::default();
            let _ = GetClientRect(hwnd, &mut rect);

            if pt.y >= 0 && pt.y < titlebar_h && pt.x >= rect.right - actions_w {
                return LRESULT(HTCLIENT as _);
            }

            return LRESULT(HTTRANSPARENT as _);
        }
        _ => {}
    }

    let prev = ORIGINAL_WNDPROC.load(Ordering::Relaxed);
    if prev != 0 {
        let prev_proc: WndProcFn = std::mem::transmute(prev);
        return CallWindowProcW(Some(prev_proc), hwnd, msg, wparam, lparam);
    }

    LRESULT(0)
}

#[cfg(target_os = "windows")]
fn apply_blank_system_cursor() {
    if let Some(raw) = blank_cursor_handle() {
        unsafe {
            use windows::Win32::UI::WindowsAndMessaging::{SetCursor, HCURSOR};
            let _ = SetCursor(Some(HCURSOR(raw as *mut _)));
        }
    }
}

#[cfg(target_os = "windows")]
fn cursor_belongs_to_window(
    root: windows::Win32::Foundation::HWND,
    cursor_hwnd: windows::Win32::Foundation::HWND,
) -> bool {
    if cursor_hwnd.0.is_null() {
        return false;
    }
    if cursor_hwnd == root {
        return true;
    }
    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::IsChild;
        IsChild(root, cursor_hwnd).as_bool()
    }
}

#[cfg(target_os = "windows")]
fn blank_cursor_handle() -> Option<isize> {
    let raw = *BLANK_CURSOR.get_or_init(|| unsafe {
        use windows::Win32::System::LibraryLoader::GetModuleHandleW;
        use windows::Win32::UI::WindowsAndMessaging::CreateCursor;

        const SIZE: i32 = 32;
        let row_bytes = ((SIZE + 15) / 16 * 2) as usize;
        let mask_len = row_bytes * SIZE as usize;
        let and_mask = vec![0xFFu8; mask_len];
        let xor_mask = vec![0u8; mask_len];
        let instance = GetModuleHandleW(None).ok().map(|m| {
            use windows::Win32::Foundation::HINSTANCE;
            HINSTANCE(m.0)
        });
        let handle = CreateCursor(
            instance,
            0,
            0,
            SIZE,
            SIZE,
            and_mask.as_ptr() as *const _,
            xor_mask.as_ptr() as *const _,
        );
        handle.map(|h| h.0 as isize).unwrap_or(0)
    });
    if raw == 0 { None } else { Some(raw) }
}

#[cfg(not(target_os = "windows"))]
fn blank_cursor_handle() -> Option<isize> {
    None
}

#[cfg(target_os = "windows")]
fn track_mouse_leave(hwnd: windows::Win32::Foundation::HWND) {
    if MOUSE_LEAVE_TRACKING.swap(true, Ordering::Relaxed) {
        return;
    }

    unsafe {
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            TrackMouseEvent, TME_LEAVE, TME_NONCLIENT, TRACKMOUSEEVENT,
        };

        let mut tme = TRACKMOUSEEVENT {
            cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
            dwFlags: TME_LEAVE | TME_NONCLIENT,
            hwndTrack: hwnd,
            dwHoverTime: 0,
        };
        let _ = TrackMouseEvent(&mut tme);
    }
}

#[cfg(target_os = "windows")]
fn emit_window_mouse_leave() {
    if let Some((app, label)) = CURSOR_EVENT_TARGET.get() {
        if let Some(window) = app.get_webview_window(label) {
            let _ = window.emit("window-mouse-leave", ());
        }
    }
}

#[cfg(target_os = "windows")]
fn cursor_inside_window(window: &WebviewWindow) -> bool {
    use windows::Win32::Foundation::{HWND, POINT, RECT};
    use windows::Win32::UI::WindowsAndMessaging::{GetCursorPos, GetWindowRect};

    let Ok(hwnd) = window.hwnd() else {
        return false;
    };

    unsafe {
        let hwnd = HWND(hwnd.0 as *mut _);
        let mut point = POINT::default();
        if GetCursorPos(&mut point).is_err() {
            return false;
        }
        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_err() {
            return false;
        }
        point.x >= rect.left
            && point.x < rect.right
            && point.y >= rect.top
            && point.y < rect.bottom
    }
}

pub fn apply_deferred(window: WebviewWindow) {
    tauri::async_runtime::spawn(async move {
        for delay_ms in [16_u64, 80, 200] {
            if delay_ms > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
            }
            maintain_chrome(&window);
        }
    });
}

pub fn start_watchdog(window: WebviewWindow) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(WATCHDOG_INTERVAL_SECS)).await;
            if window.is_visible().unwrap_or(false) {
                apply_maintain(&window);
            }
        }
    });
}

pub fn on_window_event(window: &tauri::Window, event: &tauri::WindowEvent) {
    match event {
        tauri::WindowEvent::Focused(_) => {
            if let Some(w) = window.app_handle().get_webview_window(window.label()) {
                apply_maintain(&w);
            }
        }
        tauri::WindowEvent::Resized(_) => {
            schedule_resize_maintain(window);
            if let Some(w) = window.app_handle().get_webview_window(window.label()) {
                update_scale(&w);
            }
        }
        tauri::WindowEvent::ScaleFactorChanged { .. } | tauri::WindowEvent::ThemeChanged(_) => {
            if let Some(w) = window.app_handle().get_webview_window(window.label()) {
                apply_maintain(&w);
            }
        }
        _ => {}
    }
}

fn schedule_resize_maintain(window: &tauri::Window) {
    if LAYOUT_TRANSITION.load(Ordering::Relaxed) {
        return;
    }
    let generation = RESIZE_GENERATION.fetch_add(1, Ordering::Relaxed) + 1;
    let app = window.app_handle().clone();
    let label = window.label().to_string();

    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(350)).await;
        if RESIZE_GENERATION.load(Ordering::Relaxed) != generation {
            return;
        }
        if let Some(w) = app.get_webview_window(&label) {
            apply_maintain(&w);
        }
    });
}
