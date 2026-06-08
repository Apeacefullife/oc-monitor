<p align="center">
  <a href="#readme-zh"><strong>简体中文</strong></a> · <a href="#readme-en">English</a>
</p>

<a id="readme-zh"></a>

<div align="center">

# DS-Monitor

**轻量、透明的 DeepSeek API 桌面监控工具**

实时查看余额、Token 用量与消费趋势 · 平台用量同步 · AI 智能分析

<br />

[![GitHub Repo](https://img.shields.io/badge/GitHub-milusvip%2FDS--Monitor-181717?logo=github)](https://github.com/milusvip/DS-Monitor)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows-0078D6?logo=windows)](https://github.com/milusvip/DS-Monitor)
[![Tauri](https://img.shields.io/badge/Built%20with-Tauri%202-FFC131?logo=tauri)](https://tauri.app/)
[![React](https://img.shields.io/badge/UI-React%2019-61DAFB?logo=react)](https://react.dev/)

<br />

Windows 轻量级桌面监控工具，常驻系统托盘。  
实时掌握 **DeepSeek API** 余额与 Token 消耗，同步平台用量，一键查看趋势与 AI 分析报告。

</div>

---

## 功能亮点

| 模块 | 说明 |
| :--- | :--- |
| **账户余额** | API 实时余额、可用状态、低余额红色呼吸提醒、预估可用天数 |
| **用量统计** | 当日消耗、本月消费、分模型进度条、近 7 日消耗趋势图（悬浮查看明细） |
| **平台同步** | 登录 DeepSeek 平台后后台静默同步用量，无需反复手动刷新 |
| **AI 分析** | 缓存命中率、输入 Token 构成、7 日趋势、AI 生成用量报告 |
| **桌面体验** | 无边框毛玻璃、系统托盘、窗口置顶、锁定防误触、自定义光标 |
| **个性设置** | API Key、刷新间隔、开机自启、中英文、一键清除本地数据 |

---

## 界面预览

<p align="center">
<table>
<tr><td align="center">
<table border="1" cellpadding="16" cellspacing="0">
<tr>
<td align="center" valign="top" width="50%">
<b>主面板</b><br />
<sub>余额 · 消耗 · 模型 · 趋势</sub><br /><br />
<img src="docs/screenshots/01-main.png" alt="主面板" width="320" />
</td>
<td align="center" valign="top" width="50%">
<b>趋势悬浮</b><br />
<sub>悬停柱子查看 Token 明细</sub><br /><br />
<img src="docs/screenshots/05-trend-tooltip.gif" alt="消耗趋势悬浮提示" width="320" />
</td>
</tr>
<tr>
<td align="center" valign="top" width="50%">
<b>设置</b><br />
<sub>API Key · 刷新 · 开机自启</sub><br /><br />
<img src="docs/screenshots/02-settings.png" alt="设置" width="320" />
</td>
<td align="center" valign="top" width="50%">
<b>AI 分析</b><br />
<sub>命中率 · 缓存构成 · 趋势图</sub><br /><br />
<img src="docs/screenshots/03-analysis.png" alt="AI 分析" width="320" />
</td>
</tr>
<tr>
<td align="center" valign="top" width="50%">
<b>AI 报告</b><br />
<sub>一键生成用量解读</sub><br /><br />
<img src="docs/screenshots/04-analysis-report.png" alt="AI 报告" width="320" />
</td>
<td align="center" valign="top" width="50%">
<b>右键菜单</b><br />
<sub>刷新 · 分析 · 设置 · 托盘</sub><br /><br />
<img src="docs/screenshots/06-context-menu.png" alt="右键快捷菜单" width="320" />
</td>
</tr>
</table>
</td></tr>
</table>
</p>

---

## 快速开始

### 环境要求

- Windows 10 / 11
- [Node.js](https://nodejs.org/) 18+
- [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/tools/install)（Tauri 构建）

### 安装依赖并运行

```bash
git clone https://github.com/milusvip/DS-Monitor.git
cd DS-Monitor
pnpm install
pnpm tauri dev
```

### 打包发布

```bash
pnpm tauri build
```

安装包输出目录：`src-tauri/target/release/bundle/`

---

## 使用指南

```
① 设置 API Key  →  ② 验证通过  →  ③ 平台登录（可选）  →  ④ 主面板查看数据
```

1. **配置 API Key**  
   点击右上角 ⚙️ 设置，填入 DeepSeek API Key 并验证。

2. **平台登录（推荐）**  
   用量明细需平台会话。按设置页或主界面提示登录，进入用量页后后台自动同步。

3. **日常查看**  
   - 主面板：余额、当日/本月消耗、模型用量、趋势图（悬停柱子看明细）  
   - AI 分析：命中率、缓存构成、AI 报告  
   - 托盘：关闭窗口后驻留托盘，双击或右键唤回

4. **常用操作**  
   - 标题栏：置顶 📌、锁定 🔒、最小化、关闭到托盘  
   - 右键菜单：刷新、打开分析/设置、隐藏到托盘  
   - 设置：调整刷新间隔、开机自启、切换语言

---

## 开发与构建

```bash
# 仅前端
pnpm dev

# 完整桌面应用
pnpm tauri dev

# 类型检查 + 前端构建
pnpm build
```

技术栈：**Tauri 2** · **React 19** · **TypeScript** · **Tailwind CSS 4** · **Zustand** · **ECharts**

---

## 隐私说明

- API Key 使用系统级存储加密保存在本机
- 不向第三方服务器上传 Key 或账户数据
- AI 分析报告通过你的 API Key 调用 DeepSeek 接口生成
- 「清除所有数据」可一键删除本地 Key、登录态与缓存

---

## 开源与贡献

仓库地址：**[github.com/milusvip/DS-Monitor](https://github.com/milusvip/DS-Monitor)**

欢迎 Star ⭐、提交 Issue 反馈问题、或发 Pull Request 参与贡献。

---

## 支持作者（可选）

如果 DS-Monitor 对你有帮助，欢迎自愿赞赏。  
**不赞赏也完全不影响使用。**

<p align="center">

| 微信支付 | 支付宝 |
| :---: | :---: |
| <img src="src/assets/image/wx.jpg" alt="微信赞赏" width="200" /> | <img src="src/assets/image/zfb.jpg" alt="支付宝赞赏" width="200" /> |

</p>

---

## 免责声明

本项目为**第三方开源工具**，与 DeepSeek 官方无关联，不由 DeepSeek 维护或背书。使用本软件即表示你自行承担 API 调用与账户安全相关风险。

<br />

<p align="center">
  <a href="#readme-zh">简体中文</a> · <a href="#readme-en"><strong>English</strong></a>
</p>

<a id="readme-en"></a>

<div align="center">

# DS-Monitor

**A lightweight, transparent desktop monitor for DeepSeek API usage**

Real-time balance · Token usage · Cost trends · Platform sync · AI insights

<br />

[![GitHub Repo](https://img.shields.io/badge/GitHub-milusvip%2FDS--Monitor-181717?logo=github)](https://github.com/milusvip/DS-Monitor)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows-0078D6?logo=windows)](https://github.com/milusvip/DS-Monitor)
[![Tauri](https://img.shields.io/badge/Built%20with-Tauri%202-FFC131?logo=tauri)](https://tauri.app/)
[![React](https://img.shields.io/badge/UI-React%2019-61DAFB?logo=react)](https://react.dev/)

<br />

A lightweight Windows desktop app that lives in the system tray.  
Track **DeepSeek API** balance and token usage in real time, sync platform data, and view trends plus AI analysis reports.

</div>

---

## Features

| Module | Description |
| :--- | :--- |
| **Balance** | Live API balance, availability status, low-balance alert with breathing glow, estimated days remaining |
| **Usage** | Daily & monthly spend, per-model bars, 7-day cost trend chart (hover for details) |
| **Platform sync** | Sign in to DeepSeek platform once; usage syncs silently in the background |
| **AI analysis** | Cache hit rate, input token breakdown, 7-day trends, AI-generated usage report |
| **Desktop UX** | Frameless acrylic UI, system tray, always-on-top, interaction lock, custom cursor |
| **Settings** | API Key, refresh interval, launch at startup, Chinese/English UI, clear all local data |

---

## Screenshots

<p align="center">
<table>
<tr><td align="center">
<table border="1" cellpadding="16" cellspacing="0">
<tr>
<td align="center" valign="top" width="50%">
<b>Dashboard</b><br />
<sub>Balance · spend · models · trends</sub><br /><br />
<img src="docs/screenshots/01-main.png" alt="Main dashboard" width="320" />
</td>
<td align="center" valign="top" width="50%">
<b>Trend tooltip</b><br />
<sub>Hover bars for token breakdown</sub><br /><br />
<img src="docs/screenshots/05-trend-tooltip.gif" alt="Trend chart tooltip" width="320" />
</td>
</tr>
<tr>
<td align="center" valign="top" width="50%">
<b>Settings</b><br />
<sub>API Key · refresh · auto-start</sub><br /><br />
<img src="docs/screenshots/02-settings.png" alt="Settings" width="320" />
</td>
<td align="center" valign="top" width="50%">
<b>AI analysis</b><br />
<sub>Hit rate · cache mix · charts</sub><br /><br />
<img src="docs/screenshots/03-analysis.png" alt="AI analysis" width="320" />
</td>
</tr>
<tr>
<td align="center" valign="top" width="50%">
<b>AI report</b><br />
<sub>One-click usage summary</sub><br /><br />
<img src="docs/screenshots/04-analysis-report.png" alt="AI report" width="320" />
</td>
<td align="center" valign="top" width="50%">
<b>Context menu</b><br />
<sub>Refresh · analysis · settings · tray</sub><br /><br />
<img src="docs/screenshots/06-context-menu.png" alt="Context menu" width="320" />
</td>
</tr>
</table>
</td></tr>
</table>
</p>

---

## Quick Start

### Requirements

- Windows 10 / 11
- [Node.js](https://nodejs.org/) 18+
- [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/tools/install) (for Tauri builds)

### Install & run

```bash
git clone https://github.com/milusvip/DS-Monitor.git
cd DS-Monitor
pnpm install
pnpm tauri dev
```

### Build installer

```bash
pnpm tauri build
```

Output: `src-tauri/target/release/bundle/`

---

## Usage

```
① Set API Key  →  ② Verify  →  ③ Platform login (optional)  →  ④ View dashboard
```

1. **Configure API Key**  
   Click ⚙️ Settings, enter your DeepSeek API Key, and verify.

2. **Platform login (recommended)**  
   Detailed usage requires a platform session. Sign in when prompted and open the usage page; sync continues in the background.

3. **Daily use**  
   - Dashboard: balance, daily/monthly spend, model usage, trend chart (hover bars for details)  
   - AI analysis: hit rate, cache breakdown, AI report  
   - Tray: close the window to keep running in the system tray; double-click or right-click to restore

4. **Shortcuts**  
   - Title bar: pin 📌, lock 🔒, minimize, close to tray  
   - Right-click menu: refresh, analysis, settings, hide to tray  
   - Settings: refresh interval, auto-start, language

---

## Development

```bash
# Frontend only
pnpm dev

# Full desktop app
pnpm tauri dev

# Typecheck + frontend build
pnpm build
```

**Stack:** Tauri 2 · React 19 · TypeScript · Tailwind CSS 4 · Zustand · ECharts

---

## Privacy

- API Key is encrypted and stored locally on your machine
- No Key or account data is sent to third-party servers
- AI reports are generated via your API Key through the DeepSeek API
- "Clear all data" removes local Key, session, and cache in one step

---

## Contributing

Repository: **[github.com/milusvip/DS-Monitor](https://github.com/milusvip/DS-Monitor)**

Stars ⭐, Issues, and Pull Requests are welcome.

---

## Support the Author (optional)

If DS-Monitor helps you, voluntary tips are appreciated.  
**The app works fully without tipping.**

<p align="center">

| WeChat Pay | Alipay |
| :---: | :---: |
| <img src="src/assets/image/wx.jpg" alt="WeChat tip" width="200" /> | <img src="src/assets/image/zfb.jpg" alt="Alipay tip" width="200" /> |

</p>

---

## Disclaimer

This is an **independent open-source project**. It is not affiliated with, endorsed by, or maintained by DeepSeek. You use this software at your own risk regarding API usage and account security.

---

## License

[MIT License](./LICENSE)
