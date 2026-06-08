import { invoke } from "@tauri-apps/api/core";
import { getLocale, translate } from "../i18n";

export async function syncTrayQuickMenu(): Promise<void> {
  const locale = getLocale();
  const t = (key: string) => translate(locale, key);

  try {
    await invoke("sync_tray_quick_menu", {
      labels: {
        refresh: t("quickAction.refresh"),
        analysis: t("quickAction.analysis"),
        settings: t("quickAction.settings"),
        showWindow: t("quickAction.showWindow"),
        hideWindow: t("quickAction.hideWindow"),
        unlock: t("quickAction.unlock"),
        quit: t("quickAction.quit"),
      },
    });
  } catch {
    // tray may not be ready during early startup
  }
}
