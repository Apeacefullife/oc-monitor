/// 应用全局类型定义

/** 账户状态 */
export type AccountStatus = "normal" | "low" | "exhausted" | "unknown";

/** 余额信息 */
export interface BalanceInfo {
  total_balance: number;
  granted_balance: number;
  topped_up_balance: number;
  currency?: string;
  is_available?: boolean;
  status: AccountStatus;
}

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
}

/** API 响应包装 */
export interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error: string | null;
}

/** 缓存数据 */
export interface CachedData {
  balance: BalanceInfo | null;
  daily_usage: DailyUsage[] | null;
  monthly_cost: MonthlyCost | null;
  last_updated: string | null;
}

/** 刷新间隔选项 */
export type RefreshInterval = 30 | 60 | 120 | 300;

/** 应用设置 */
export interface AppSettings {
  api_key: string;
  refresh_interval: RefreshInterval;
  auto_start: boolean;
  dark_mode: boolean;
  language: "zh" | "en";
}

/** 应用状态 */
export interface AppState {
  // 数据
  balance: BalanceInfo | null;
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
  usageSource: "platform" | null;

  // 设置
  settings: Partial<AppSettings>;
  settingsOpen: boolean;
  analysisOpen: boolean;

  // 操作
  getApiKey: () => Promise<string | null>;
  fetchData: () => Promise<void>;
  applySilentRefresh: (payload: {
    balance?: BalanceInfo | null;
    usage?: {
      daily: DailyUsage[];
      models: DailyUsage[];
      monthly: MonthlyCost & { total_tokens?: number; request_count?: number };
      has_daily_granularity: boolean;
    } | null;
  }) => void;
  setBalance: (balance: BalanceInfo) => void;
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
