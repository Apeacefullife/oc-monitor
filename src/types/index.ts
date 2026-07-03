/// 应用全局类型定义

/** 数据源选项 */
export type DataSource = "opencode" | "claude";

/** 每日用量 */
export interface DailyUsage {
  date: string;
  total_tokens: number;
  input_tokens: number;
  input_cache_hit_tokens?: number;
  input_cache_miss_tokens?: number;
  output_tokens: number;
  request_count: number;
  cost: number;
  model: string;
}

/** 月度消费 */
export interface MonthlyCost {
  total_cost: number;
  currency: string;
  month: string;
  request_count?: number;
  total_tokens?: number;
}

/** API 响应包装 */
export interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error: string | null;
}

/** 归一化用量 */
export interface NormalizedUsage {
  daily: DailyUsage[];
  models: DailyUsage[];
  monthly: MonthlyCost & { total_tokens?: number; request_count?: number };
  has_daily_granularity: boolean;
}

/** 缓存数据 */
export interface CachedData {
  daily_usage: DailyUsage[] | null;
  model_usage: DailyUsage[] | null;
  monthly_cost: MonthlyCost | null;
  platform_usage: NormalizedUsage | null;
  last_updated: string | null;
}

/** 刷新间隔选项 */
export type RefreshInterval = 30 | 60 | 120 | 300;

/** 应用设置 */
export interface AppSettings {
  refresh_interval: RefreshInterval;
  auto_start: boolean;
  dark_mode: boolean;
  language: "zh" | "en";
  selected_models: string[];
  data_source?: DataSource;
}

/** 应用状态 */
export interface AppState {
  // 数据
  dailyUsage: DailyUsage[];
  modelUsage: DailyUsage[];
  monthlyCost: MonthlyCost | null;
  lastUpdated: string | null;
  loading: boolean;
  error: string | null;
  usageCurrency: string;
  hasDailyGranularity: boolean;
  hasUsageData: boolean;
  usageUnavailable: boolean;
  usageSource: "local" | null;

  // 设置
  settings: Partial<AppSettings>;
  settingsOpen: boolean;
  analysisOpen: boolean;

  // 操作
  fetchData: () => Promise<void>;
  applySilentRefresh: (payload: { usage?: NormalizedUsage | null }) => void;
  setDailyUsage: (usage: DailyUsage[]) => void;
  setModelUsage: (usage: DailyUsage[]) => void;
  setMonthlyCost: (cost: MonthlyCost) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  setLastUpdated: (time: string) => void;
  restoreFromCache: () => Promise<void>;
  toggleSettings: () => void;
  toggleAnalysis: () => void;
  updateSettings: (settings: Partial<AppSettings>) => void;
}
