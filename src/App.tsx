import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { useT, translate, useI18nStore } from "./i18n";
import { useAppStore } from "./stores/useAppStore";
import { useWindowStore } from "./stores/useWindowStore";
import TitleBar from "./components/TitleBar";
import MainPanel from "./components/MainPanel";
import Settings from "./components/Settings";
import AiAnalysisPanel from "./components/AiAnalysis";
import StatusIndicator from "./components/common/StatusIndicator";
import LoadingSpinner from "./components/common/LoadingSpinner";
import QuickActionMenu, {
  type QuickActionMenuState,
} from "./components/common/QuickActionMenu";
import type { BalanceInfo, DailyUsage } from "./types";
import {
  animateDrawerClose,
  animateDrawerOpen,
  clearLegacyCanvasPref,
  setWindowLogicalSize,
} from "./utils/analysisLayout";
import PlatformLoginBanner, {
  type PlatformBannerMode,
} from "./components/PlatformLoginBanner";
import {
  hasPlatformSession,
  openPlatformLogin,
  openPlatformLoginIfNeeded,
} from "./utils/platformLogin";

const BASE_WIDTH = 360;
const PANEL_ANIM_MS = 320;
const MIN_WINDOW_HEIGHT = 260;
const MAX_WINDOW_HEIGHT = 720;

interface SilentUsage {
  daily: DailyUsage[];
  models: DailyUsage[];
  monthly: {
    total_cost: number;
    currency: string;
    month: string;
    total_tokens?: number;
    request_count?: number;
  };
  has_daily_granularity: boolean;
}

interface SilentRefreshPayload {
  balance?: BalanceInfo | null;
  usage?: SilentUsage | null;
}

function App() {
  const t = useT();
  const locale = useI18nStore((s) => s.locale);
  const {
    balance,
    dailyUsage,
    modelUsage,
    monthlyCost,
    lastUpdated,
    loading,
    error,
    settingsOpen,
    analysisOpen,
    fetchData,
    restoreFromCache,
    applySilentRefresh,
    getApiKey,
    usageCurrency,
    hasDailyGranularity,
  } = useAppStore();

  const shellRef = useRef<HTMLDivElement>(null);
  const mainColRef = useRef<HTMLDivElement>(null);
  const analysisColRef = useRef<HTMLElement>(null);
  const settingsColRef = useRef<HTMLElement>(null);
  const lastSizeRef = useRef({ width: 0, height: 0 });
  const panelTransitionRef = useRef(false);

  const [analysisMounted, setAnalysisMounted] = useState(false);
  const [analysisVisible, setAnalysisVisible] = useState(false);
  const [settingsMounted, setSettingsMounted] = useState(false);
  const [settingsVisible, setSettingsVisible] = useState(false);
  const [platformBanner, setPlatformBanner] =
    useState<PlatformBannerMode | null>(null);
  const [quickActionMenu, setQuickActionMenu] =
    useState<QuickActionMenuState | null>(null);
  const setInteractionLocked = useWindowStore((s) => s.setInteractionLocked);

  const measureShellHeight = useCallback(() => {
    const mainHeight = mainColRef.current?.offsetHeight ?? 0;
    if (mainHeight <= 0) return 0;
    return Math.min(
      MAX_WINDOW_HEIGHT,
      Math.max(MIN_WINDOW_HEIGHT, Math.ceil(mainHeight)),
    );
  }, []);

  const syncWindowSize = useCallback(async () => {
    if (!mainColRef.current || panelTransitionRef.current) return;

    const height = measureShellHeight();
    if (height <= 0) return;

    if (
      lastSizeRef.current.width === BASE_WIDTH &&
      lastSizeRef.current.height === height
    ) {
      return;
    }
    lastSizeRef.current = { width: BASE_WIDTH, height };
    await setWindowLogicalSize(BASE_WIDTH, height);
  }, [measureShellHeight, loading, error, balance, hasDailyGranularity]);

  const closeAnalysisDrawer = useCallback(async () => {
    if (!analysisOpen) return;
    await animateDrawerClose(
      setAnalysisVisible,
      setAnalysisMounted,
      () => analysisColRef.current,
      PANEL_ANIM_MS,
    );
    useAppStore.setState({ analysisOpen: false });
  }, [analysisOpen]);

  const closeSettingsDrawer = useCallback(async () => {
    if (!settingsOpen) return;
    await animateDrawerClose(
      setSettingsVisible,
      setSettingsMounted,
      () => settingsColRef.current,
      PANEL_ANIM_MS,
    );
    useAppStore.setState({ settingsOpen: false });
  }, [settingsOpen]);

  const openAnalysisDrawer = useCallback(async () => {
    if (analysisOpen) return;
    if (settingsOpen) {
      await closeSettingsDrawer();
    }
    useAppStore.setState({ analysisOpen: true, settingsOpen: false });
    await animateDrawerOpen(
      setAnalysisMounted,
      setAnalysisVisible,
      () => analysisColRef.current,
      PANEL_ANIM_MS,
    );
  }, [analysisOpen, settingsOpen, closeSettingsDrawer]);

  const openSettingsDrawer = useCallback(async () => {
    if (settingsOpen) return;
    if (analysisOpen) {
      await closeAnalysisDrawer();
    }
    useAppStore.setState({ settingsOpen: true, analysisOpen: false });
    await animateDrawerOpen(
      setSettingsMounted,
      setSettingsVisible,
      () => settingsColRef.current,
      PANEL_ANIM_MS,
    );
  }, [analysisOpen, settingsOpen, closeAnalysisDrawer]);

  const handleToggleAnalysis = useCallback(async () => {
    if (panelTransitionRef.current) return;
    panelTransitionRef.current = true;
    try {
      if (analysisOpen) {
        await closeAnalysisDrawer();
      } else {
        await openAnalysisDrawer();
      }
    } finally {
      panelTransitionRef.current = false;
    }
  }, [analysisOpen, closeAnalysisDrawer, openAnalysisDrawer]);

  const handleToggleSettings = useCallback(async () => {
    if (panelTransitionRef.current) return;
    panelTransitionRef.current = true;
    try {
      if (settingsOpen) {
        await closeSettingsDrawer();
      } else {
        await openSettingsDrawer();
      }
    } finally {
      panelTransitionRef.current = false;
    }
  }, [settingsOpen, closeSettingsDrawer, openSettingsDrawer]);

  const handlePlatformLogin = useCallback(async () => {
    try {
      await openPlatformLogin();
      setPlatformBanner("waiting");
    } catch (err) {
      console.error("open_platform_login failed:", err);
    }
  }, []);

  const refreshPlatformBanner = useCallback(async () => {
    const key = await getApiKey();
    if (!key) {
      setPlatformBanner(null);
      return;
    }
    const connected = await hasPlatformSession();
    if (connected) {
      setPlatformBanner(null);
      return;
    }
    setPlatformBanner((prev) => (prev === "expired" ? "expired" : "required"));
  }, [getApiKey]);

  useEffect(() => {
    clearLegacyCanvasPref();
    void restoreFromCache();
    void useWindowStore.getState().syncInteractionLocked();

    const startupTimer = window.setTimeout(() => {
      void getApiKey().then(async (key) => {
        if (key) {
          void invoke("silent_refresh").catch(() => fetchData());

          window.setTimeout(async () => {
            const result = await openPlatformLoginIfNeeded();
            if (result === "opened") {
              setPlatformBanner("waiting");
            } else if (result === "has_session") {
              setPlatformBanner(null);
            } else {
              await refreshPlatformBanner();
            }
          }, 1000);
        } else {
          void openSettingsDrawer();
        }
      });
    }, 400);

    const unsubs: Array<() => void> = [];

    listen<SilentRefreshPayload>("silent-refresh-done", (event) => {
      applySilentRefresh({
        balance: event.payload.balance ?? null,
        usage: event.payload.usage ?? null,
      });
    }).then((fn) => unsubs.push(fn));

    listen("tray-open-settings", () => {
      void openSettingsDrawer();
    }).then((fn) => unsubs.push(fn));

    listen("tray-open-analysis", () => {
      void openAnalysisDrawer();
    }).then((fn) => unsubs.push(fn));

    listen<boolean>("interaction-lock-changed", (event) => {
      setInteractionLocked(event.payload);
    }).then((fn) => unsubs.push(fn));

    listen("platform-login-opening", () => {
      setPlatformBanner("waiting");
    }).then((fn) => unsubs.push(fn));

    listen("platform-login-done", () => {
      setPlatformBanner(null);
    }).then((fn) => unsubs.push(fn));

    listen("platform-session-expired", () => {
      setPlatformBanner("expired");
    }).then((fn) => unsubs.push(fn));

    listen("all-data-cleared", () => {
      setPlatformBanner(null);
    }).then((fn) => unsubs.push(fn));

    listen("platform-login-cancelled", () => {
      void hasPlatformSession().then((connected) => {
        if (connected) {
          setPlatformBanner(null);
          return;
        }
        setPlatformBanner((prev) =>
          prev === "expired" ? "expired" : "required",
        );
      });
    }).then((fn) => unsubs.push(fn));

    return () => {
      window.clearTimeout(startupTimer);
      unsubs.forEach((fn) => fn());
    };
  }, []);

  useEffect(() => {
    const node = mainColRef.current;
    if (!node) return;

    let raf = 0;
    const scheduleSync = () => {
      cancelAnimationFrame(raf);
      raf = requestAnimationFrame(() => {
        void syncWindowSize();
      });
    };

    const observer = new ResizeObserver(scheduleSync);
    observer.observe(node);
    scheduleSync();

    return () => {
      cancelAnimationFrame(raf);
      observer.disconnect();
    };
  }, [syncWindowSize]);

  const needsApiKey = error === "errors.apiKeyRequired";
  const errorMessage =
    error && error.startsWith("errors.")
      ? translate(locale, error)
      : error;
  const overlayVisible = analysisVisible || settingsVisible;
  const interactionLocked = useWindowStore((s) => s.interactionLocked);

  const handleContextMenu = useCallback(
    (event: React.MouseEvent<HTMLDivElement>) => {
      if ((event.target as HTMLElement).closest(".titlebar")) return;
      event.preventDefault();
      setQuickActionMenu({ x: event.clientX, y: event.clientY });
    },
    [],
  );

  const handleQuickRefresh = useCallback(() => {
    void invoke("silent_refresh").catch(() => fetchData());
  }, [fetchData]);

  const handleQuickOpenAnalysis = useCallback(() => {
    void openAnalysisDrawer();
  }, [openAnalysisDrawer]);

  const handleQuickOpenSettings = useCallback(() => {
    void openSettingsDrawer();
  }, [openSettingsDrawer]);

  const handleQuickHideToTray = useCallback(() => {
    void invoke("hide_main_window");
  }, []);

  const handleQuickUnlock = useCallback(() => {
    void invoke("set_window_interaction_locked", { locked: false }).then(() => {
      setInteractionLocked(false);
    });
  }, [setInteractionLocked]);

  return (
    <div
      ref={shellRef}
      className={`app-shell relative overflow-hidden font-sans select-none${
        interactionLocked ? " app-shell--interaction-locked" : ""
      }`}
      onContextMenu={handleContextMenu}
    >
      <div
        ref={mainColRef}
        className="app-main-col flex flex-col w-[360px] h-fit"
      >
        <TitleBar />

        <div className="flex items-center justify-between px-4 py-1.5 shrink-0">
          <StatusIndicator
            loading={loading}
            lastUpdated={lastUpdated}
            status={balance?.status}
          />
          <div className="flex items-center gap-0.5">
            <button
              type="button"
              onClick={() => void handleToggleAnalysis()}
              className={`transition-colors p-1 rounded-lg ${
                analysisOpen
                  ? "text-deepseek-400 bg-deepseek-500/15"
                  : "text-white/30 hover:text-white/60 hover:bg-white/5"
              }`}
              title={t("toolbar.aiAnalysis")}
            >
              <svg
                className="w-3.5 h-3.5"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
                strokeWidth={2}
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  d="M9.813 15.904L9 18.75l-.813-2.846a4.5 4.5 0 00-3.09-3.09L2.25 12l2.846-.813a4.5 4.5 0 003.09-3.09L9 5.25l.813 2.846a4.5 4.5 0 003.09 3.09L15.75 12l-2.846.813a4.5 4.5 0 00-3.09 3.09zM18.259 8.715L18 9.75l-.259-1.035a3.375 3.375 0 00-2.455-2.456L14.25 6l1.036-.259a3.375 3.375 0 002.455-2.456L18 2.25l.259 1.035a3.375 3.375 0 002.456 2.456L21.75 6l-1.035.259a3.375 3.375 0 00-2.456 2.456z"
                />
              </svg>
            </button>
            <button
              type="button"
              onClick={() => void handleToggleSettings()}
              className={`transition-colors p-1 rounded-lg ${
                settingsOpen
                  ? "text-deepseek-400 bg-deepseek-500/15"
                  : "text-white/30 hover:text-white/60 hover:bg-white/5"
              }`}
              title={t("toolbar.settings")}
            >
              <svg
                className="w-3.5 h-3.5"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
                strokeWidth={2}
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
                />
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                />
              </svg>
            </button>
          </div>
        </div>

        {error && !needsApiKey && (
          <div className="mx-4 mb-2 px-3 py-2 rounded-xl bg-danger/10 border border-danger/20 text-danger text-xs shrink-0">
            {errorMessage}
          </div>
        )}

        {needsApiKey && (
          <div className="mx-4 mb-2 px-3 py-3 rounded-xl glass-card text-center shrink-0">
            <p className="text-white/50 text-xs mb-2">{t("apiKey.configurePrompt")}</p>
            <button
              type="button"
              onClick={() => void openSettingsDrawer()}
              className="px-4 py-1.5 rounded-lg bg-deepseek-500/20 text-deepseek-400 text-xs hover:bg-deepseek-500/30 transition-colors"
            >
              {t("apiKey.openSettings")}
            </button>
          </div>
        )}

        {platformBanner && !needsApiKey && (
          <PlatformLoginBanner
            mode={platformBanner}
            onLogin={() => void handlePlatformLogin()}
          />
        )}

        <div className="px-4 pb-3 space-y-3 shrink-0">
          {loading && !balance ? (
            <div className="flex items-center justify-center h-40">
              <LoadingSpinner />
            </div>
          ) : (
            <MainPanel
              balance={balance}
              dailyUsage={dailyUsage}
              modelUsage={modelUsage}
              monthlyCost={monthlyCost}
              loading={loading}
              usageCurrency={usageCurrency}
              hasDailyGranularity={hasDailyGranularity}
            />
          )}
        </div>
      </div>

      {(analysisMounted || settingsMounted) && (
        <button
          type="button"
          className={`analysis-backdrop ${overlayVisible ? "analysis-backdrop--visible" : ""}`}
          onClick={() => {
            if (analysisOpen) void handleToggleAnalysis();
            else if (settingsOpen) void handleToggleSettings();
          }}
          aria-label={
            analysisOpen ? t("toolbar.closeAnalysis") : t("toolbar.closeSettings")
          }
        />
      )}

      {analysisMounted && (
        <AiAnalysisPanel
          ref={analysisColRef}
          visible={analysisVisible}
          onClose={() => void handleToggleAnalysis()}
          balance={balance}
          dailyUsage={dailyUsage}
          monthlyCost={monthlyCost}
          usageCurrency={usageCurrency}
        />
      )}

      {settingsMounted && (
        <Settings
          ref={settingsColRef}
          visible={settingsVisible}
          onClose={() => void handleToggleSettings()}
        />
      )}

      <QuickActionMenu
        state={quickActionMenu}
        locked={interactionLocked}
        onClose={() => setQuickActionMenu(null)}
        onRefresh={handleQuickRefresh}
        onOpenAnalysis={handleQuickOpenAnalysis}
        onOpenSettings={handleQuickOpenSettings}
        onHideToTray={handleQuickHideToTray}
        onUnlock={handleQuickUnlock}
      />
    </div>
  );
}

export default App;
