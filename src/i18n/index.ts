import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { syncTrayQuickMenu } from "../utils/trayMenu";
import { en } from "./locales/en";
import { zh, type MessageTree } from "./locales/zh";

export type Locale = "zh" | "en";

const messages: Record<Locale, MessageTree> = { zh, en };

function resolvePath(tree: MessageTree, path: string): string | undefined {
  const parts = path.split(".");
  let node: unknown = tree;
  for (const part of parts) {
    if (node === null || typeof node !== "object" || !(part in node)) {
      return undefined;
    }
    node = (node as Record<string, unknown>)[part];
  }
  return typeof node === "string" ? node : undefined;
}

function interpolate(
  template: string,
  params?: Record<string, string | number>,
) {
  if (!params) return template;
  return template.replace(/\{\{(\w+)\}\}/g, (_, key: string) =>
    String(params[key] ?? ""),
  );
}

export function translate(
  locale: Locale,
  key: string,
  params?: Record<string, string | number>,
): string {
  const text =
    resolvePath(messages[locale], key) ??
    resolvePath(messages.zh, key) ??
    key;
  return interpolate(text, params);
}

interface I18nState {
  locale: Locale;
  initialized: boolean;
  setLocale: (locale: Locale) => void;
  initLocale: () => Promise<void>;
}

function detectSystemLocale(): Locale {
  const lang = navigator.language.toLowerCase();
  return lang.startsWith("zh") ? "zh" : "en";
}

export const useI18nStore = create<I18nState>((set, get) => ({
  locale: detectSystemLocale(),
  initialized: false,

  setLocale: (locale) => {
    set({ locale });
    document.documentElement.lang = locale === "zh" ? "zh-CN" : "en";
    void invoke("save_setting", { key: "language", value: locale }).catch(
      () => {},
    );
    void syncTrayQuickMenu();
  },

  initLocale: async () => {
    if (get().initialized) return;

    let locale = detectSystemLocale();
    try {
      const saved = await invoke<string | null>("get_setting", {
        key: "language",
      });
      if (saved === "zh" || saved === "en") {
        locale = saved;
      }
    } catch {
      // use detected locale
    }

    document.documentElement.lang = locale === "zh" ? "zh-CN" : "en";
    set({ locale, initialized: true });
    void syncTrayQuickMenu();
  },
}));

export function useT() {
  const locale = useI18nStore((s) => s.locale);
  return (key: string, params?: Record<string, string | number>) =>
    translate(locale, key, params);
}

export function getLocale(): Locale {
  return useI18nStore.getState().locale;
}
