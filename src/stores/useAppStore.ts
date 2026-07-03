import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type {
  AppState,
  DataSource,
  DailyUsage,
  MonthlyCost,
  RawUsageRecord,
} from "../types";
import { useSettingsStore } from "./useSettingsStore";
import { aggregateUsage, filterByDataSource } from "../utils/usageAggregate";

function emptyUsage() {
  return {
    daily: [] as DailyUsage[],
    models: [] as DailyUsage[],
    monthly: {
      total_cost: 0,
      currency: "USD",
      month: new Date().toISOString().slice(0, 7),
      total_tokens: 0,
      request_count: 0,
    },
    has_daily_granularity: false,
  };
}

function applyUsageToUpdates(usage: ReturnType<typeof aggregateUsage>) {
  return {
    dailyUsage: usage.daily,
    modelUsage: usage.models,
    usageCurrency: usage.monthly.currency || "USD",
    hasDailyGranularity: usage.has_daily_granularity,
    hasUsageData:
      usage.models.length > 0 ||
      usage.daily.some((d) => d.total_tokens > 0 || d.cost > 0) ||
      usage.monthly.total_cost > 0 ||
      (usage.monthly.total_tokens ?? 0) > 0,
    monthlyCost: usage.monthly.total_cost > 0 || (usage.monthly.total_tokens ?? 0) > 0
      ? ({
          total_cost: usage.monthly.total_cost,
          currency: usage.monthly.currency,
          month: usage.monthly.month,
          request_count: usage.monthly.request_count,
          total_tokens: usage.monthly.total_tokens,
        } as MonthlyCost)
      : null,
  };
}

export const useAppStore = create<AppState>((set) => ({
  rawRecords: [],
  dailyUsage: [],
  modelUsage: [],
  monthlyCost: null,
  lastUpdated: null,
  loading: false,
  error: null,
  usageCurrency: "USD",
  hasDailyGranularity: true,
  hasUsageData: false,
  usageUnavailable: true,
  usageSource: null,

  settings: {
    refresh_interval: 60,
    auto_start: false,
    dark_mode: true,
    language: "zh",
  },
  settingsOpen: false,
  analysisOpen: false,

  fetchData: async () => {
    set({ loading: true, error: null });
    try {
      // 始终拉全量原始记录；dataSource 完全在前端处理
      await invoke("silent_refresh");
    } catch (err) {
      set({ error: String(err), loading: false });
    }
  },

  applySilentRefresh: (payload) => {
    const records = payload.raw_records ?? [];
    // 用当前 dataSource 算一次默认聚合，存到 store（让老组件 / 兜底组件能读）
    const ds = useSettingsStore.getState().dataSource;
    const usage = records.length > 0
      ? aggregateUsage(filterByDataSource(records, ds))
      : emptyUsage();
    set({
      rawRecords: records,
      lastUpdated: new Date().toISOString(),
      loading: false,
      error: null,
      usageSource: "local",
      ...applyUsageToUpdates(usage),
    });
  },

  /** 不重新拉数据，仅按指定 dataSource 重算聚合（切换瞬时生效） */
  recomputeForDataSource: (ds: DataSource) => {
    const { rawRecords } = useAppStore.getState();
    const usage = rawRecords.length > 0
      ? aggregateUsage(filterByDataSource(rawRecords, ds))
      : emptyUsage();
    set({
      ...applyUsageToUpdates(usage),
      lastUpdated: new Date().toISOString(),
    });
  },

  setDailyUsage: (usage) => set({ dailyUsage: usage }),
  setModelUsage: (usage) => set({ modelUsage: usage }),
  setMonthlyCost: (cost) => set({ monthlyCost: cost }),
  setLoading: (loading) => set({ loading }),
  setError: (error) => set({ error }),
  setLastUpdated: (time) => set({ lastUpdated: time }),

  restoreFromCache: async () => {
    try {
      const cached = await invoke<{
        daily_usage: DailyUsage[] | null;
        model_usage: DailyUsage[] | null;
        monthly_cost: MonthlyCost | null;
        platform_usage: unknown;
        raw_records: RawUsageRecord[] | null;
        last_updated: string | null;
      } | null>("get_cached_data");

      if (cached?.raw_records && cached.raw_records.length > 0) {
        const ds = useSettingsStore.getState().dataSource;
        const filtered = filterByDataSource(cached.raw_records, ds);
        const usage = aggregateUsage(filtered);
        set({
          rawRecords: cached.raw_records,
          lastUpdated: cached.last_updated ?? null,
          usageSource: "local",
          ...applyUsageToUpdates(usage),
        });
      } else if (cached) {
        // 兼容旧缓存（只有聚合数据、没有 raw_records）
        const monthly =
          cached.monthly_cost && cached.monthly_cost.total_cost > 0
            ? cached.monthly_cost
            : null;
        const daily = cached.daily_usage ?? [];
        const models = cached.model_usage ?? [];
        const hasCachedUsage =
          models.some((m) => m.total_tokens > 0 || m.cost > 0) ||
          daily.some((d) => d.total_tokens > 0 || d.cost > 0) ||
          (monthly?.total_cost ?? 0) > 0;
        set({
          rawRecords: [],
          dailyUsage: daily,
          modelUsage: models,
          monthlyCost: monthly,
          lastUpdated: cached.last_updated ?? null,
          hasUsageData: hasCachedUsage,
          usageUnavailable: true,
          usageSource: "local",
        });
      }
    } catch {
      // 缓存不是必须的
    }
  },

  toggleSettings: () =>
    set((state) => ({
      settingsOpen: !state.settingsOpen,
      analysisOpen: false,
    })),

  toggleAnalysis: () =>
    set((state) => ({
      analysisOpen: !state.analysisOpen,
      settingsOpen: false,
    })),

  updateSettings: (newSettings) =>
    set((state) => ({
      settings: { ...state.settings, ...newSettings },
    })),
}));
