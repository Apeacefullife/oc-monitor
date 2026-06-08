/** DeepSeek API 端点 */
export const API_BASE_URL = "https://api.deepseek.com";

/** 刷新间隔选项（秒） */
export const REFRESH_INTERVAL_VALUES = [30, 60, 120, 300] as const;

/** 默认刷新间隔（秒） */
export const DEFAULT_REFRESH_INTERVAL = 60;

/** 开源仓库 */
export const GITHUB_REPO_URL = "https://github.com/milusvip/DS-Monitor";

/** 缓存键名 */
export const CACHE_KEYS = {
  BALANCE: "cache_balance",
  DAILY_USAGE: "cache_daily_usage",
  MONTHLY_COST: "cache_monthly_cost",
  LAST_UPDATED: "cache_last_updated",
  API_KEY: "api_key",
  REFRESH_INTERVAL: "refresh_interval",
  AUTO_START: "auto_start",
} as const;

/** 模型名称（API / 平台 canonical id → 显示名） */
export const MODELS = {
  "deepseek-v4-flash": "DeepSeek V4 Flash",
  "deepseek-v4-pro": "DeepSeek V4 Pro",
  "deepseek-chat": "DeepSeek V4 Flash",
  "deepseek-reasoner": "DeepSeek V4 Pro",
} as const;

/** 余额阈值 */
export const BALANCE_THRESHOLDS = {
  LOW: 5.0, // 低于此值视为低余额
  CRITICAL: 1.0, // 低于此值视为危险
} as const;
