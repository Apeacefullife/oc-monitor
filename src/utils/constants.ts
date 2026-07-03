/** 刷新间隔选项（秒） */
export const REFRESH_INTERVAL_VALUES = [30, 60, 120, 300] as const;

/** 默认刷新间隔（秒） */
export const DEFAULT_REFRESH_INTERVAL = 60;

/** 开源仓库 */
export const GITHUB_REPO_URL = "https://github.com/Apeacefullife/oc-monitor";

/** 缓存键名 */
export const CACHE_KEYS = {
  DAILY_USAGE: "cache_daily_usage",
  MONTHLY_COST: "cache_monthly_cost",
  LAST_UPDATED: "cache_last_updated",
  REFRESH_INTERVAL: "refresh_interval",
  AUTO_START: "auto_start",
} as const;
