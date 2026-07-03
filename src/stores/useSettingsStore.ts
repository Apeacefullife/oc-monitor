import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { RefreshInterval } from "../types";
import { DEFAULT_REFRESH_INTERVAL } from "../utils/constants";
import { TRACKED_MODEL_IDS } from "../utils/modelUsage";
import { useAppStore } from "./useAppStore";

const STORAGE_KEY = "oc_monitor_selected_models";

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

interface SettingsState {
  refreshInterval: number;
  autoStart: boolean;
  darkMode: boolean;
  selectedModels: string[];
  settingsInitialized: boolean;

  setRefreshInterval: (interval: number) => void;
  setAutoStart: (enabled: boolean) => void;
  setDarkMode: (enabled: boolean) => void;
  setSelectedModels: (models: string[]) => void;
  toggleModel: (modelId: string) => void;
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
