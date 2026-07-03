import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { useT } from "./i18n";
import { useAppStore } from "./stores/useAppStore";
import TitleBar from "./components/TitleBar";
import MainPanel from "./components/MainPanel";
import Settings from "./components/Settings";
import AiAnalysisPanel from "./components/AiAnalysis";
import StatusIndicator from "./components/common/StatusIndicator";
import LoadingSpinner from "./components/common/LoadingSpinner";
import QuickActionMenu, {
  type QuickActionMenuState,
} from "./components/common/QuickActionMenu";
import type { RawUsageRecord } from "./types";
import {
  animateDrawerClose,
  animateDrawerOpen,
  clearLegacyCanvasPref,
  setWindowLogicalSize,
} from "./utils/analysisLayout";

const BASE_WIDTH = 360;
const PANEL_ANIM_MS = 320;
const MIN_WINDOW_HEIGHT = 260;
const MAX_WINDOW_HEIGHT = 720;

interface SilentRefreshPayload {
  raw_records?: RawUsageRecord[] | null;
}

function App() {
  const t = useT();
  const {
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
  const [quickActionMenu, setQuickActionMenu] =
    useState<QuickActionMenuState | null>(null);

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
  }, [measureShellHeight, loading, error, hasDailyGranularity]);

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

  useEffect(() => {
    clearLegacyCanvasPref();
    // 启动时清缓存，确保读到全量数据
    void invoke("clear_cache").finally(() => {
      void restoreFromCache();
    });

    const startupTimer = window.setTimeout(() => {
      void invoke("silent_refresh");
    }, 400);

    const unsubs: Array<() => void> = [];

    listen<SilentRefreshPayload>("silent-refresh-done", (event) => {
      applySilentRefresh({
        raw_records: event.payload.raw_records ?? null,
      });
    }).then((fn) => unsubs.push(fn));

    listen("tray-open-settings", () => {
      void openSettingsDrawer();
    }).then((fn) => unsubs.push(fn));

    listen("tray-open-analysis", () => {
      void openAnalysisDrawer();
    }).then((fn) => unsubs.push(fn));

    listen("all-data-cleared", () => {
      // reset handled by settings
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

  const errorMessage = error ?? null;
  const overlayVisible = analysisVisible || settingsVisible;

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

  return (
    <div
      ref={shellRef}
      className="app-shell relative overflow-hidden font-sans select-none"
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

        {error && (
          <div className="mx-4 mb-2 px-3 py-2 rounded-xl bg-danger/10 border border-danger/20 text-danger text-xs shrink-0">
            {errorMessage}
          </div>
        )}

        <div className="px-4 pb-3 space-y-3 shrink-0">
          {loading && dailyUsage.length === 0 ? (
            <div className="flex items-center justify-center h-40">
              <LoadingSpinner />
            </div>
          ) : (
            <MainPanel
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
        onClose={() => setQuickActionMenu(null)}
        onRefresh={handleQuickRefresh}
        onOpenAnalysis={handleQuickOpenAnalysis}
        onOpenSettings={handleQuickOpenSettings}
        onHideToTray={handleQuickHideToTray}
      />
    </div>
  );
}

export default App;
