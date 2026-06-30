import { forwardRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useSettingsStore } from "../../stores/useSettingsStore";
import { useAppStore } from "../../stores/useAppStore";
import { useI18nStore, useT, type Locale } from "../../i18n";
import { GITHUB_REPO_URL, REFRESH_INTERVAL_VALUES } from "../../utils/constants";
import ToggleSwitch from "../common/ToggleSwitch";

interface Props {
  visible: boolean;
  onClose: () => void;
}

const LANG_OPTIONS: { value: Locale; labelKey: "settings.languageZh" | "settings.languageEn" }[] =
  [
    { value: "zh", labelKey: "settings.languageZh" },
    { value: "en", labelKey: "settings.languageEn" },
  ];

function refreshLabel(value: number, t: ReturnType<typeof useT>) {
  if (value < 60) return t("settings.refreshSeconds", { n: value });
  return t("settings.refreshMinutes", { n: value / 60 });
}

export default forwardRef<HTMLElement, Props>(function Settings(
  { visible, onClose },
  ref,
) {
  const t = useT();
  const locale = useI18nStore((s) => s.locale);
  const setLocale = useI18nStore((s) => s.setLocale);

  const {
    refreshInterval,
    autoStart,
    applyAutoStart,
    applyRefreshInterval,
  } = useSettingsStore();

  const [clearing, setClearing] = useState(false);
  const [clearConfirmOpen, setClearConfirmOpen] = useState(false);
  const [clearError, setClearError] = useState<string | null>(null);
  const [autoStartError, setAutoStartError] = useState<string | null>(null);
  const [autoStartBusy, setAutoStartBusy] = useState(false);

  const handleRefreshIntervalChange = (interval: number) => {
    void applyRefreshInterval(interval);
  };

  const handleAutoStartChange = async (enabled: boolean) => {
    if (autoStartBusy) return;

    setAutoStartBusy(true);
    setAutoStartError(null);

    try {
      await applyAutoStart(enabled);
    } catch (err) {
      setAutoStartError(String(err));
    } finally {
      setAutoStartBusy(false);
    }
  };

  const handleLanguageChange = (lang: Locale) => {
    setLocale(lang);
    useAppStore.getState().updateSettings({ language: lang });
  };

  const handleClearAll = async () => {
    setClearing(true);
    setClearError(null);
    try {
      await invoke("clear_all_data");
      useSettingsStore.getState().reset();
      setClearConfirmOpen(false);
      useAppStore.setState({
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
      });
    } catch (err) {
      setClearError(String(err));
    } finally {
      setClearing(false);
    }
  };

  return (
    <aside
      ref={ref}
      className={`analysis-drawer analysis-drawer-right settings-drawer ${
        visible ? "analysis-drawer--visible" : ""
      }`}
      role="dialog"
      aria-modal="true"
      aria-labelledby="settings-title"
      aria-hidden={!visible}
    >
      <div className="analysis-drawer-panel flex flex-col min-h-0 h-full">
        <div className="settings-ui">
          <header className="settings-ui-head">
            <h2 id="settings-title" className="settings-ui-title">
              {t("settings.title")}
            </h2>
            <button
              type="button"
              onClick={onClose}
              className="settings-ui-close"
              aria-label={t("settings.collapse")}
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
                  d="M9 5l7 7-7 7"
                />
              </svg>
            </button>
          </header>

          <div className="settings-ui-body">
            <section className="settings-ui-block">
              <div className="settings-ui-caption">
                {t("settings.dataSource")}
              </div>
              <p className="settings-ui-note">{t("settings.dataSourceHint")}</p>
            </section>

              <div className="settings-ui-divider" />

            <section className="settings-ui-block settings-ui-block--grow">
              <div>
                <label className="settings-ui-caption">
                  {t("settings.refreshInterval")}
                </label>
                <div className="settings-ui-segment">
                  {REFRESH_INTERVAL_VALUES.map((option) => (
                    <button
                      key={option}
                      type="button"
                      onClick={() => handleRefreshIntervalChange(option)}
                      className={`settings-ui-segment-btn ${
                        refreshInterval === option
                          ? "settings-ui-segment-btn--active"
                          : ""
                      }`}
                    >
                      {refreshLabel(option, t)}
                    </button>
                  ))}
                </div>
              </div>

              <div>
                <div className="settings-ui-row">
                  <label
                    htmlFor="settings-auto-start"
                    className="settings-ui-row-label"
                  >
                    {t("settings.autoStart")}
                  </label>
                  <ToggleSwitch
                    id="settings-auto-start"
                    checked={autoStart}
                    disabled={autoStartBusy}
                    onCheckedChange={(enabled) =>
                      void handleAutoStartChange(enabled)
                    }
                    ariaLabel={t("settings.autoStart")}
                  />
                </div>
                {autoStartError && (
                  <p className="settings-ui-note settings-ui-note--err">
                    {autoStartError}
                  </p>
                )}
              </div>

              <div className="settings-ui-row">
                <span className="settings-ui-row-label">
                  {t("settings.language")}
                </span>
                <div className="settings-ui-lang">
                  {LANG_OPTIONS.map((option) => (
                    <button
                      key={option.value}
                      type="button"
                      onClick={() => handleLanguageChange(option.value)}
                      className={`settings-ui-lang-btn ${
                        locale === option.value
                          ? "settings-ui-lang-btn--active"
                          : ""
                      }`}
                    >
                      {t(option.labelKey)}
                    </button>
                  ))}
                </div>
              </div>

              <div className="settings-ui-row">
                <span className="settings-ui-row-label">
                  {t("settings.github")}
                </span>
                <button
                  type="button"
                  onClick={() => void openUrl(GITHUB_REPO_URL)}
                  className="settings-ui-link settings-ui-link--icon"
                  aria-label={t("settings.githubOpen")}
                >
                  <svg
                    className="settings-ui-link-icon"
                    viewBox="0 0 24 24"
                    fill="currentColor"
                    aria-hidden
                  >
                    <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0 0 24 12c0-6.63-5.37-12-12-12z" />
                  </svg>
                  {t("settings.githubOpen")}
                </button>
              </div>
              <p className="settings-ui-note">{t("settings.githubHint")}</p>
            </section>
          </div>

          <footer
            className={`settings-ui-foot ${
              clearConfirmOpen ? "settings-ui-foot--confirm" : ""
            }`}
          >
            {clearConfirmOpen ? (
              <>
                <p className="settings-ui-confirm-text">
                  {t("settings.clearAllConfirm")}
                </p>
                {clearError && (
                  <p className="settings-ui-note settings-ui-note--err">
                    {clearError}
                  </p>
                )}
                <div className="settings-ui-confirm-actions">
                  <button
                    type="button"
                    onClick={() => {
                      setClearConfirmOpen(false);
                      setClearError(null);
                    }}
                    disabled={clearing}
                    className="settings-ui-confirm-cancel"
                  >
                    {t("settings.clearAllCancel")}
                  </button>
                  <button
                    type="button"
                    onClick={() => void handleClearAll()}
                    disabled={clearing}
                    className="settings-ui-confirm-ok"
                  >
                    {clearing
                      ? t("settings.clearAllClearing")
                      : t("settings.clearAllConfirmBtn")}
                  </button>
                </div>
              </>
            ) : (
              <>
                <button
                  type="button"
                  onClick={() => setClearConfirmOpen(true)}
                  className="settings-ui-danger"
                >
                  {t("settings.clearAllBtn")}
                </button>
                <span className="settings-ui-version">{t("app.version")}</span>
              </>
            )}
          </footer>
        </div>
      </div>
    </aside>
  );
});
