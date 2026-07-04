<p align="center">
  <a href="README.md">简体中文</a> · <strong>English</strong>
</p>

<div align="center">

# OC-Monitor

**A lightweight, transparent desktop monitor for OpenCode / Claude Code token usage**

Dual data sources · one-click data-source switch · real-time trends · AI insights

<br />

[![GitHub Repo](https://img.shields.io/badge/GitHub-Apeacefullife%2Foc--monitor-181717?logo=github)](https://github.com/Apeacefullife/oc-monitor)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
[![Platform: Windows](https://img.shields.io/badge/Platform-Windows-0078D6?logo=windows)](https://github.com/Apeacefullife/oc-monitor)
[![Tauri](https://img.shields.io/badge/Built%20with-Tauri%202-FFC131?logo=tauri)](https://tauri.app/)
[![React](https://img.shields.io/badge/UI-React%2019-61DAFB?logo=react)](https://react.dev/)

<br />

A lightweight Windows desktop app that lives in the system tray.  
Pulls from both `~/.claude/projects/` (Claude Code's own JSONL logs) and `~/.cc-switch/cc-switch.db` (CCSwitch proxy records). A **"Data source" toggle in Settings** lets you switch between the two views instantly — OpenCode mode, Claude Code mode (including Claude Code CLI pointed at DeepSeek / OpenCode Go / Anthropic). No disk re-read on switch.

</div>

---

> Forked and re-scoped from [DS-Monitor](https://github.com/milusvip/DS-Monitor), shifting focus from DeepSeek API balance tracking to local OpenCode / Claude Code call analytics.

---

## Use cases

| How you use your AI tools | What OC-Monitor will show |
| :--- | :--- |
| OpenCode CLI + CCSwitch proxy | Switch to **OpenCode** mode — all records with `provider_id = "_opencode_session"` in CCSwitch |
| Claude Code CLI → Anthropic API | Switch to **Claude Code** mode — auto-reads from `~/.claude/projects/` JSONL |
| Claude Code CLI + `ANTHROPIC_BASE_URL` pointing at DeepSeek / OpenCode Go / etc. | Switch to **Claude Code** mode — JSONL rows with `model=deepseek-chat` etc. are **fully tracked** (pricing table is built in) |
| OpenCode + Claude Code used side by side | Toggle between data sources to compare the two side by side |

> 💡 The two sources are **naturally de-duplicated** — CCSwitch rows are tagged `_opencode_session`, JSONL rows are tagged `_claude_log`. They never overlap.

---

## Features

| Module | Description |
| :--- | :--- |
| **Dual data sources** | Backend pulls CCSwitch SQLite + Claude Code JSONL in one pass; frontend filters by dataSource toggle |
| **Data source switch** | OpenCode / Claude Code toggle in Settings — **pure-frontend useMemo**, instant, no backend call |
| **Token stats** | Full breakdown of Input / Cache-Read / Cache-Creation / Output tokens, per-model |
| **Cost trends** | Today / month spend, 7-day cost trend chart with hoverable per-bar details |
| **Cache hit rate** | Auto-computed hit rate with prompt-stability hints |
| **AI analysis** | Hit-rate trend, token composition, 7-day curves, **one-click AI usage report** (uses your own API key) |
| **Model filter** | Pick which models to show on the main panel, at least one required |
| **Desktop UX** | Frameless acrylic UI, system tray, always-on-top, interaction lock, custom cursor |
| **Local-first privacy** | **Fully offline** — all data parsed on-device, **nothing is uploaded to any external server** |
| **Settings** | Data source, model filter, refresh interval, launch at startup, Chinese / English UI, clear all local data |

---

## What data does it read?

OC-Monitor parses your OpenCode / Claude Code usage **entirely on-device**. It never modifies any source files. The backend pulls from **two sources**, merges them, and the frontend filters by the "Data source" toggle in Settings.

| Source | Path | What it contains | Toggle to use |
| :--- | :--- | :--- | :--- |
| CCSwitch SQLite | `~/.cc-switch/cc-switch.db` | All rows of the `proxy_request_logs` table (every `provider_id`) | **OpenCode** — shows only rows with `provider_id = "_opencode_session"` |
| Claude Code JSONL | `~/.claude/projects/**/*.jsonl` | Messages with `type=assistant` and `message.role=assistant`, reading the `usage` block (input / cache_read / cache_creation / output tokens, model, timestamp); tagged with `provider_id = "_claude_log"` | **Claude Code** — covers usage from running Claude Code CLI directly against **any** endpoint (DeepSeek, OpenCode Go, Anthropic, etc.) |

Read-only. Works fully offline.

Switching the "Data source" toggle is **a pure-frontend operation** (instant) — no disk re-read, no backend invocation.

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
① Launch OC-Monitor  →  ② Auto-scan local usage  →  ③ View dashboard  →  ④ Toggle data source  →  ⑤ Generate AI report
```

1. **First launch**  
   OC-Monitor automatically scans `~/.claude/projects/` and `~/.cc-switch/`. No account setup required.

2. **Data source switch (the core feature)**  
   - Settings → Data source → choose **OpenCode** / **Claude Code**  
   - The dashboard numbers **update instantly**, no disk re-read, no spinner  
   - **OpenCode mode** shows CCSwitch rows with `provider_id = "_opencode_session"`  
   - **Claude Code mode** shows JSONL rows from `~/.claude/projects/**/*.jsonl` (covers Claude Code CLI against any endpoint)

3. **Daily use**  
   - Dashboard: today's / this month's spend, per-model share, 7-day trend (hover bars for details)  
   - AI analysis: hit rate, cache breakdown, AI report  
   - Tray: close the window to keep running in the system tray; double-click or right-click to restore

4. **Shortcuts**  
   - Title bar: pin 📌, lock 🔒, minimize, close to tray  
   - Right-click menu: refresh, analysis, settings, hide to tray  
   - Settings: data source, model filter, refresh interval, auto-start, language

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
