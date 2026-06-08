# 截图取景指南

将截图保存到本目录，文件名需与下表一致，主 README 会自动引用（趋势悬浮为 GIF，其余为 PNG）。

## 通用建议

- **格式**：PNG（清晰、适合 GitHub）
- **背景**：使用你日常的桌面壁纸即可，能体现毛玻璃效果
- **侧栏截图**（设置 / AI 分析）：窗口会向右变宽，请截**整个应用窗口**，不要只截左侧 360px
- **清晰度**：Windows 截图工具（`Win + Shift + S`）或 Snipping Tool 均可

---

## 逐张说明

### `01-main.png` — 主面板

- 已配置 API Key，有余额与用量数据
- 默认窄窗（约 360px 宽）
- 尽量包含：余额卡、当日/本月消耗、模型条、趋势图

### `02-settings.png` — 设置

- 点击主界面右上角 ⚙️ 打开设置侧栏
- 截全窗，右侧应能看到：API Key、平台登录、刷新间隔、开机自启、语言、GitHub

### `03-analysis.png` — AI 分析（图表）

- 点击主界面右上角 ✨ 打开分析侧栏
- 确保可见：今日/近 7 日/本月命中率、命中率趋势图、输入 Token 构成图

### `04-analysis-report.png` — AI 分析（报告）

- 在分析侧栏顶部点击「刷新/生成」按钮，等待 AI 报告生成
- 截报告展开后的状态（可与 `03` 相同窗口，重点是报告区域）

### `05-trend-tooltip.gif` — 消耗趋势悬浮提示（GIF）

- 在主面板底部 **消耗趋势** 图表区域
- 录制鼠标移入/移出柱子的过程，展示 tooltip 弹出与切换
- 建议：3～5 秒、宽 360px 左右、循环播放
- 可用 Windows 自带录屏或 [ScreenToGif](https://www.screentogif.com/) 导出 GIF
- 保存为 `docs/screenshots/05-trend-tooltip.gif`（若已有 `.png` 可删除）

### `06-context-menu.png` — 右键快捷菜单

- 在主界面内容区空白处 **右键**
- 菜单应完整可见：刷新、AI 分析、设置、隐藏到托盘
- 默认窄窗（约 360px 宽）即可

---

## 检查清单

- [ ] `01-main.png`
- [ ] `02-settings.png`
- [ ] `03-analysis.png`
- [ ] `04-analysis-report.png`
- [ ] `05-trend-tooltip.gif`
- [ ] `06-context-menu.png`

全部就位后，推送到 GitHub，README 中的图片会自动显示。
