@echo off
chcp 65001 >nul
title DeepSeek Monitor - Dev Server

cd /d "%~dp0"

echo ╔══════════════════════════════════════════╗
echo ║     DeepSeek Monitor - Dev Server        ║
║     Tauri 2.0 + React + TypeScript       ║
echo ╚══════════════════════════════════════════╝
echo.

:: 检查 pnpm
where pnpm >nul 2>&1
if %errorlevel% neq 0 (
    echo [错误] 未找到 pnpm，请先安装 Node.js 和 pnpm
    pause
    exit /b 1
)

echo [1/3] 安装前端依赖...
call pnpm install
if %errorlevel% neq 0 (
    echo [错误] 依赖安装失败
    pause
    exit /b 1
)
echo [OK] 依赖安装完成
echo.

echo [2/3] 启动 Tauri 开发服务器...
echo.
echo 提示：首次启动会编译 Rust 后端，需要一些时间
echo 成功后会自动弹出应用窗口
echo 按 Ctrl+C 停止服务器
echo.
call pnpm tauri dev

pause
