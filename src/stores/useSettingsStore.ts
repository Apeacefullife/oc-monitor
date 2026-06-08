import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { RefreshInterval } from "../types";
import { DEFAULT_REFRESH_INTERVAL } from "../utils/constants";
import { useAppStore } from "./useAppStore";

interface SettingsState {
  apiKey: string;
  apiKeyVerified: boolean;
  refreshInterval: number;
  autoStart: boolean;
  darkMode: boolean;
  settingsInitialized: boolean;

  setApiKey: (key: string) => void;
  setApiKeyVerified: (verified: boolean) => void;
  setRefreshInterval: (interval: number) => void;
  setAutoStart: (enabled: boolean) => void;
  setDarkMode: (enabled: boolean) => void;
  initSettings: () => Promise<void>;
  applyAutoStart: (enabled: boolean) => Promise<void>;
  applyRefreshInterval: (interval: number) => Promise<void>;

  reset: () => void;
}

const initialState = {
  apiKey: "",
  apiKeyVerified: false,
  refreshInterval: DEFAULT_REFRESH_INTERVAL,
  autoStart: false,
  darkMode: true,
};

export const useSettingsStore = create<SettingsState>((set, get) => ({
  ...initialState,
  settingsInitialized: false,

  setApiKey: (key) => set({ apiKey: key, apiKeyVerified: false }),
  setApiKeyVerified: (verified) => set({ apiKeyVerified: verified }),
  setRefreshInterval: (interval) => set({ refreshInterval: interval }),
  setAutoStart: (enabled) => set({ autoStart: enabled }),
  setDarkMode: (enabled) => set({ darkMode: enabled }),

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

  reset: () => set({ ...initialState, settingsInitialized: true }),
}));
