/// Windows 开机自启管理
///
/// 通过写入注册表实现：
/// `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`

/// 注册开机自启
pub fn enable() -> Result<(), String> {
    let exe_path = std::env::current_exe().map_err(|e| format!("获取程序路径失败: {}", e))?;
    let path_str = exe_path.to_string_lossy().to_string();

    set_registry_value(&path_str).map_err(|e| format!("注册开机自启失败: {}", e))?;
    Ok(())
}

/// 取消开机自启
pub fn disable() -> Result<(), String> {
    delete_registry_value().map_err(|e| format!("取消开机自启失败: {}", e))?;
    Ok(())
}

/// 检查是否已启用开机自启
pub fn is_enabled() -> bool {
    get_registry_value().is_some()
}

#[cfg(target_os = "windows")]
fn set_registry_value(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    let path_key = r"Software\Microsoft\Windows\CurrentVersion\Run";
    let key = hkcu.open_subkey_with_flags(
        path_key,
        winreg::enums::KEY_SET_VALUE,
    )?;
    key.set_value("OC-Monitor", &path)?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn delete_registry_value() -> Result<(), Box<dyn std::error::Error>> {
    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    let path_key = r"Software\Microsoft\Windows\CurrentVersion\Run";
    let key = hkcu.open_subkey_with_flags(
        path_key,
        winreg::enums::KEY_SET_VALUE,
    )?;
    let _ = key.delete_value("OC-Monitor");
    Ok(())
}

#[cfg(target_os = "windows")]
fn get_registry_value() -> Option<String> {
    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    let path_key = r"Software\Microsoft\Windows\CurrentVersion\Run";
    let key = hkcu.open_subkey_with_flags(path_key, winreg::enums::KEY_READ).ok()?;
    key.get_value::<String, _>("OC-Monitor").ok()
}

#[cfg(not(target_os = "windows"))]
fn set_registry_value(_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // macOS/Linux：通过 launchd/systemd 实现
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn delete_registry_value() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn get_registry_value() -> Option<String> {
    None
}
