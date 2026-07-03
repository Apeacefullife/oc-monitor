import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { DataSource, RefreshInterval } from "../types";
import { DEFAULT_REFRESH_INTERVAL } from "../utils/constants";
import { TRACKED_MODEL_IDS } from "../utils/modelUsage";
import { useAppStore } from "./useAppStore";

const STORAGE_KEY = "oc_monitor_selected_models";
const DATA_SOURCE_KEY = "oc_monitor_data_source";

function loadSelectedModels(): string[] {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);
      if (Array.isArray(parsed) && parsed.length > 0) return parsed;
    }
  } catch {}
  return [...TRACKED_MODEL_IDS];
}

function saveSelectedModels(models: string[]) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(models));
}

function loadDataSource(): DataSource {
  try {
    const v = localStorage.getItem(DATA_SOURCE_KEY);
    if (v === "opencode" || v === "claude") return v;
  } catch {}
  return "opencode";
}

function saveDataSource(s: DataSource) {
  try {
    localStorage.setItem(DATA_SOURCE_KEY, s);
  } catch {}
}

interface SettingsState {
  refreshInterval: number;
  autoStart: boolean;
  darkMode: boolean;
  selectedModels: string[];
  dataSource: DataSource;
  settingsInitialized: boolean;

  setRefreshInterval: (interval: number) => void;
  setAutoStart: (enabled: boolean) => void;
  setDarkMode: (enabled: boolean) => void;
  setSelectedModels: (models: string[]) => void;
  toggleModel: (modelId: string) => void;
  setDataSource: (source: DataSource) => void;
  applyDataSource: (source: DataSource) => Promise<void>;
  initSettings: () => Promise<void>;
  applyAutoStart: (enabled: boolean) => Promise<void>;
  applyRefreshInterval: (interval: number) => Promise<void>;

  reset: () => void;
}

const initialState = {
  refreshInterval: DEFAULT_REFRESH_INTERVAL,
  autoStart: true,
  darkMode: true,
  selectedModels: loadSelectedModels(),
  dataSource: loadDataSource(),
};

export const useSettingsStore = create<SettingsState>((set, get) => ({
  ...initialState,
  settingsInitialized: false,

  setRefreshInterval: (interval) => set({ refreshInterval: interval }),
  setAutoStart: (enabled) => set({ autoStart: enabled }),
  setDarkMode: (enabled) => set({ darkMode: enabled }),

  setSelectedModels: (models) => {
    saveSelectedModels(models);
    set({ selectedModels: models });
  },

  toggleModel: (modelId) => {
    const current = get().selectedModels;
    const next = current.includes(modelId)
      ? current.filter((m) => m !== modelId)
      : [...current, modelId];
    if (next.length === 0) return;
    saveSelectedModels(next);
    set({ selectedModels: next });
  },

  setDataSource: (source) => {
    saveDataSource(source);
    set({ dataSource: source });
    // 瞬时切换：仅用已有 rawRecords 重算聚合，不重新 invoke 后端
    useAppStore.getState().recomputeForDataSource(source);
  },

  applyDataSource: async (source) => {
    const previous = get().dataSource;
    saveDataSource(source);
    set({ dataSource: source });
    try {
      // 不再 invoke 后端，dataSource 完全在前端处理
      useAppStore.getState().recomputeForDataSource(source);
    } catch (err) {
      set({ dataSource: previous });
      saveDataSource(previous);
      throw err;
    }
  },

  initSettings: async () => {
    if (get().settingsInitialized) return;
    try {
      const [autoStart, refreshInterval] = await Promise.all([
        invoke<boolean>("get_auto_start"),
        invoke<number>("get_refresh_interval"),
      ]);
      set({
        autoStart,
        refreshInterval,
        settingsInitialized: true,
      });
      useAppStore.getState().updateSettings({
        auto_start: autoStart,
        refresh_interval: refreshInterval as RefreshInterval,
        data_source: get().dataSource,
      });
    } catch {
      set({ settingsInitialized: true });
    }
  },

  applyAutoStart: async (enabled) => {
    const previous = get().autoStart;
    set({ autoStart: enabled });
    try {
      await invoke("set_auto_start", { enabled });
      useAppStore.getState().updateSettings({ auto_start: enabled });
    } catch (err) {
      set({ autoStart: previous });
      throw err;
    }
  },

  applyRefreshInterval: async (interval) => {
    const previous = get().refreshInterval;
    set({ refreshInterval: interval });
    try {
      await invoke("set_refresh_interval", { secs: interval });
      useAppStore.getState().updateSettings({
        refresh_interval: interval as RefreshInterval,
      });
    } catch (err) {
      set({ refreshInterval: previous });
      throw err;
    }
  },

  reset: () => {
    localStorage.removeItem(STORAGE_KEY);
    set({ ...initialState, selectedModels: [...TRACKED_MODEL_IDS], settingsInitialized: true });
  },
}));
