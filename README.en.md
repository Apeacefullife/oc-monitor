<p align="center">
  <a href="README.md">简体中文</a> · <strong>English</strong>
</p>

<div align="center">

# OC-Monitor

**A lightweight, transparent desktop monitor for OpenCode / Claude Code token usage**

Real-time token tracking · model breakdown · cost trends · AI insights

<br />

[![GitHub Repo](https://img.shields.io/badge/GitHub-Apeacefullife%2Foc--monitor-181717?logo=github)](https://github.com/Apeacefullife/oc-monitor)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows-0078D6?logo=windows)](https://github.com/Apeacefullife/oc-monitor)
[![Tauri](https://img.shields.io/badge/Built%20with-Tauri%202-FFC131?logo=tauri)](https://tauri.app/)
[![React](https://img.shields.io/badge/UI-React%2019-61DAFB?logo=react)](https://react.dev/)

<br />

A lightweight Windows desktop app that lives in the system tray.  
Track **OpenCode / Claude Code** token usage, model distribution and cost trends in real time, with one-click AI usage reports.

</div>

---

> Forked and re-scoped from [DS-Monitor](https://github.com/milusvip/DS-Monitor), shifting focus from DeepSeek API balance tracking to local OpenCode / Claude Code call analytics.

---

## Features

| Module | Description |
| :--- | :--- |
| **Dual data sources** | Reads Claude Code JSONL logs from `~/.claude/projects/` **and** the active OpenCode provider records from `~/.cc-switch/cc-switch.db` |
| **Token stats** | Live aggregation of Input / Cache-Read / Cache-Creation / Output tokens with per-model breakdown |
| **Cost trends** | Daily & monthly spend, 7-day cost trend chart (hover for details) |
| **AI analysis** | Cache hit rate, token composition, 7-day trend, one-click AI usage report |
| **Desktop UX** | Frameless acrylic UI, system tray, always-on-top, interaction lock, custom cursor |
| **Local-first privacy** | All data parsed locally — **nothing is uploaded to third-party servers** |
| **Settings** | Refresh interval, launch at startup, Chinese / English UI, clear all local data |

---

## What data does it read?

OC-Monitor parses your OpenCode / Claude Code usage **entirely on-device**. It never modifies any source files.

| Source | Path | Content |
| :--- | :--- | :--- |
| Claude Code JSONL | `~/.claude/projects/**/*.jsonl` | The `usage` field of assistant messages (input / cache_read / cache_creation / output tokens, model, timestamp) |
| CCSwitch SQLite | `~/.cc-switch/cc-switch.db` | The `proxy_request_logs` table, filtered to the active provider for the current `app_type` (default: `_opencode_session`) |

Read-only. Works fully offline.

---

## Screenshots

<p align="center">
<table>
<tr><td align="center">
<table border="1" cellpadding="16" cellspacing="0">
<tr>
<td align="center" valign="top" width="50%">
<b>Dashboard</b><br />
<sub>Usage · models · trends</sub><br /><br />
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
<sub>Refresh · auto-start · language</sub><br /><br />
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
- A local install of [Claude Code](https://docs.anthropic.com/en/docs/claude-code) or [OpenCode](https://opencode.ai/) with usage history

### Install & run

```bash
git clone https://github.com/Apeacefullife/oc-monitor.git
cd oc-monitor
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
① Launch OC-Monitor  →  ② Auto-scan local usage  →  ③ View dashboard  →  ④ Generate AI report
```

1. **First launch**  
   OC-Monitor automatically scans `~/.claude/projects/` and `~/.cc-switch/`. No account setup required.

2. **Daily use**  
   - Dashboard: today's / this month's spend, per-model share, 7-day trend (hover bars for details)  
   - AI analysis: hit rate, cache breakdown, AI report  
   - Tray: close the window to keep running in the system tray; double-click or right-click to restore

3. **Shortcuts**  
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

**Stack:** Tauri 2 · React 19 · TypeScript · Tailwind CSS 4 · Zustand · ECharts · rusqlite

---

## Privacy

- All usage data is **parsed locally on your machine** — nothing is uploaded to external servers
- "Clear all data" removes the local cache and session in one step
- This is an independent third-party tool, not affiliated with Anthropic, OpenCode or CCSwitch

---

## Contributing

Repository: **[github.com/Apeacefullife/oc-monitor](https://github.com/Apeacefullife/oc-monitor)**

Stars ⭐, Issues, and Pull Requests are welcome.

---

## Credits

- Architecture & UX inspired by [milusvip/DS-Monitor](https://github.com/milusvip/DS-Monitor)
- UI powered by [Tauri 2](https://tauri.app/) and [React 19](https://react.dev/)
- Charts by [Apache ECharts](https://echarts.apache.org/)

---

## License

[MIT License](./LICENSE)
