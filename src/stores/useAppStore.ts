import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type {
  AppState,
  BalanceInfo,
  DailyUsage,
  MonthlyCost,
} from "../types";

interface NormalizedUsage {
  daily: DailyUsage[];
  models: DailyUsage[];
  monthly: MonthlyCost & { total_tokens?: number; request_count?: number };
  has_daily_granularity: boolean;
}

function hasMeaningfulUsage(usage: NormalizedUsage | null | undefined): boolean {
  if (!usage) return false;
  return (
    usage.models.length > 0 ||
    usage.daily.some((d) => d.total_tokens > 0 || d.cost > 0) ||
    usage.monthly.total_cost > 0 ||
    (usage.monthly.total_tokens ?? 0) > 0
  );
}

function applyUsageToUpdates(
  usage: NormalizedUsage,
  updates: Partial<AppState>,
): void {
  updates.dailyUsage = usage.daily;
  updates.modelUsage = usage.models;
  updates.usageCurrency = usage.monthly.currency || "CNY";
  updates.hasDailyGranularity = usage.has_daily_granularity;
  updates.hasUsageData = true;
  updates.usageSource = "platform";

  if (
    usage.monthly.total_cost > 0 ||
    (usage.monthly.total_tokens ?? 0) > 0
  ) {
    updates.monthlyCost = {
      total_cost: usage.monthly.total_cost,
      currency: usage.monthly.currency,
      month: usage.monthly.month,
      request_count: usage.monthly.request_count,
    };
  } else if (usage.models.some((m) => m.total_tokens > 0 || m.cost > 0)) {
    updates.monthlyCost = {
      total_cost: usage.models.reduce((s, m) => s + m.cost, 0),
      currency: usage.monthly.currency,
      month: usage.monthly.month,
      request_count:
        usage.monthly.request_count ??
        usage.models.reduce((s, m) => s + (m.request_count ?? 0), 0),
    };
  } else {
    updates.monthlyCost = null;
  }
}

function applyUsageToState(usage: NormalizedUsage): Partial<AppState> {
  const updates: Partial<AppState> = {};
  applyUsageToUpdates(usage, updates);
  return updates;
}

function syncTrayTooltip(state: AppState): Promise<void> {
  return invoke("update_tray_tooltip", {
    balance: state.balance?.total_balance ?? null,
    currency: state.balance?.currency ?? null,
    monthlyCost: state.monthlyCost?.total_cost ?? null,
    usageCurrency: state.usageCurrency ?? state.monthlyCost?.currency ?? null,
  }).then(() => {}) as Promise<void>;
}

export const useAppStore = create<AppState>((set, get) => ({
  balance: null,
  dailyUsage: [],
  modelUsage: [],
  monthlyCost: null,
  lastUpdated: null,
  loading: false,
  error: null,
  usageCurrency: "CNY",
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

  getApiKey: async (): Promise<string | null> => {
    try {
      return await invoke<string | null>("get_api_key");
    } catch (err) {
      console.error("get_api_key failed:", err);
      return null;
    }
  },

  fetchData: async () => {
    const apiKey = await get().getApiKey();
    if (!apiKey) {
      set({ error: "errors.apiKeyRequired", loading: false });
      return;
    }

    set({ loading: true, error: null });

    try {
      await invoke("silent_refresh");
    } catch (err) {
      set({ error: String(err), loading: false });
    }
  },

  applySilentRefresh: (payload: {
    balance?: BalanceInfo | null;
    usage?: NormalizedUsage | null;
  }) => {
    const updates: Partial<AppState> = {
      lastUpdated: new Date().toISOString(),
      loading: false,
      error: null,
    };
    if (payload.balance) {
      updates.balance = payload.balance;
    }
    if (hasMeaningfulUsage(payload.usage)) {
      applyUsageToUpdates(payload.usage!, updates);
    }
    set(updates as AppState);
    syncTrayTooltip({ ...get(), ...updates } as AppState);
  },

  setBalance: (balance) => set({ balance }),
  setDailyUsage: (usage) => set({ dailyUsage: usage }),
  setModelUsage: (usage) => set({ modelUsage: usage }),
  setMonthlyCost: (cost) => set({ monthlyCost: cost }),
  setLoading: (loading) => set({ loading }),
  setError: (error) => set({ error }),
  setLastUpdated: (time) => set({ lastUpdated: time }),

  restoreFromCache: async () => {
    try {
      const cached = await invoke<{
        balance: BalanceInfo | null;
        daily_usage: DailyUsage[] | null;
        model_usage: DailyUsage[] | null;
        monthly_cost: MonthlyCost | null;
        platform_usage: NormalizedUsage | null;
        last_updated: string | null;
      } | null>("get_cached_data");

      const platformFromCache =
        cached?.platform_usage && hasMeaningfulUsage(cached.platform_usage)
          ? cached.platform_usage
          : null;

      if (cached) {
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

        if (platformFromCache) {
          set({
            balance: cached.balance ?? null,
            ...applyUsageToState(platformFromCache),
            lastUpdated: cached.last_updated ?? null,
            usageUnavailable: true,
          });
        } else {
          set({
            balance: cached.balance ?? null,
            dailyUsage: daily,
            modelUsage: models,
            monthlyCost: monthly,
            lastUpdated: cached.last_updated ?? null,
            hasUsageData: hasCachedUsage,
            usageUnavailable: true,
          });
        }
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
