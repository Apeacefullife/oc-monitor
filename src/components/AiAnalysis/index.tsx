import { forwardRef, useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useI18nStore, useT } from "../../i18n";
import type { DailyUsage, MonthlyCost } from "../../types";
import {
  buildUsageAnalysisSnapshot,
  formatHitRate,
} from "../../utils/usageAnalysis";
import { formatCurrency, formatTokens } from "../../utils/format";
import HitRateTrendChart from "./HitRateTrendChart";
import CacheTokenChart from "./CacheTokenChart";
import AiReportView from "./AiReportView";

const PANEL_ANIM_MS = 320;

interface Props {
  visible: boolean;
  onClose: () => void;
  dailyUsage: DailyUsage[];
  monthlyCost: MonthlyCost | null;
  usageCurrency: string;
}

export default forwardRef<HTMLElement, Props>(function AiAnalysisPanel(
  {
    visible,
    onClose,
    dailyUsage,
    monthlyCost,
    usageCurrency,
  },
  ref,
) {
  const t = useT();
  const locale = useI18nStore((s) => s.locale);

  const snapshot = useMemo(
    () =>
      buildUsageAnalysisSnapshot(
        dailyUsage,
        monthlyCost,
        usageCurrency,
        locale,
      ),
    [dailyUsage, monthlyCost, usageCurrency, locale],
  );

  const [aiText, setAiText] = useState<string | null>(null);
  const [aiLoading, setAiLoading] = useState(false);
  const [aiError, setAiError] = useState<string | null>(null);
  const [chartsReady, setChartsReady] = useState(false);
  const [reportExpanded, setReportExpanded] = useState(false);

  useEffect(() => {
    if (!visible) {
      setChartsReady(false);
      setReportExpanded(false);
      setAiText(null);
      setAiError(null);
      return;
    }
    const timer = window.setTimeout(() => {
      setChartsReady(true);
    }, PANEL_ANIM_MS);
    return () => window.clearTimeout(timer);
  }, [visible]);

  useEffect(() => {
    setAiText(null);
    setAiError(null);
    setReportExpanded(false);
  }, [locale]);

  const fetchAiAnalysis = useCallback(async () => {
    setAiLoading(true);
    setAiError(null);

    const payload = JSON.stringify({
      monthly_cost: snapshot.summary.monthlyCost,
      cache_hit_rate: {
        today: snapshot.today.hitRate,
        last_7_days: snapshot.last7Days.hitRate,
        month: snapshot.month.hitRate,
      },
      tokens_7d: snapshot.summary.totalTokens7d,
      cost_7d: snapshot.summary.totalCost7d,
      output_ratio: snapshot.summary.outputRatio,
      daily_trend: snapshot.dailyTrend.map((d) => ({
        date: d.date,
        hit_rate: d.hitRate,
      })),
      local_insights: snapshot.insights.slice(0, 2),
      locale,
    });

    try {
      const res = await invoke<{ success: boolean; content?: string; error?: string }>(
        "analyze_usage_ai",
        { payload, locale },
      );
      if (res.success && res.content) {
        setAiText(res.content);
        setReportExpanded(true);
      } else {
        setAiError(res.error ?? t("errors.aiAnalysisFailed"));
      }
    } catch (err) {
      setAiError(String(err));
    } finally {
      setAiLoading(false);
    }
  }, [snapshot, locale, t]);

  return (
    <aside
      ref={ref}
      className={`analysis-drawer analysis-drawer-right ${
        visible ? "analysis-drawer--visible" : ""
      }`}
      aria-hidden={!visible}
      aria-label={t("analysis.ariaLabel")}
    >
      <div className="analysis-drawer-panel px-3 pt-2 pb-3 flex flex-col flex-1 min-h-0 h-full">
        <div className="flex items-center justify-end gap-0.5 shrink-0">
          <button
            type="button"
            onClick={() => void fetchAiAnalysis()}
            disabled={aiLoading}
            className="w-6 h-6 flex items-center justify-center rounded-md text-white/35 hover:text-white/65 hover:bg-white/5 transition-colors disabled:opacity-40"
            title={t("analysis.refresh")}
            aria-label={t("analysis.refresh")}
          >
            <svg
              className={`w-3.5 h-3.5 ${aiLoading ? "animate-spin" : ""}`}
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              strokeWidth={2}
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0l3.181 3.183a8.25 8.25 0 0013.803-3.7M4.031 9.865a8.25 8.25 0 0113.803-3.7l3.181 3.182"
              />
            </svg>
          </button>
          <button
            type="button"
            onClick={onClose}
            className="w-6 h-6 flex items-center justify-center rounded-md text-white/35 hover:text-white/65 hover:bg-white/5 transition-colors"
            title={t("analysis.collapse")}
            aria-label={t("analysis.collapse")}
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
        </div>

        <div
          className={`analysis-charts-section shrink-0 space-y-2 mt-2 ${
            reportExpanded ? "analysis-charts-section--collapsed" : ""
          }`}
        >
          <div className="analysis-metric-card">
            <div className="grid grid-cols-3 gap-1 text-center">
              {[
                { label: t("analysis.today"), stats: snapshot.today },
                { label: t("analysis.last7Days"), stats: snapshot.last7Days },
                { label: t("analysis.thisMonth"), stats: snapshot.month },
              ].map(({ label, stats }) => (
                <div key={label} className="analysis-rate-cell">
                  <div className="text-[10px] ui-muted">{label}</div>
                  <div className="text-[15px] font-semibold text-white tabular-nums mt-0.5">
                    {stats.input > 0 ? formatHitRate(stats.hitRate) : "--"}
                  </div>
                </div>
              ))}
            </div>
          </div>

          {chartsReady && snapshot.dailyTrend.length > 0 && (
            <>
              <HitRateTrendChart data={snapshot.dailyTrend} />
              <CacheTokenChart data={snapshot.dailyTrend} />
            </>
          )}

          <div className="analysis-metric-card text-[10px] leading-relaxed text-white/80 tabular-nums">
            <span className="text-white">
              {formatTokens(snapshot.summary.totalTokens7d)}
            </span>
            <span className="ui-muted"> · </span>
            <span className="text-white">
              {formatCurrency(snapshot.summary.totalCost7d, usageCurrency)}
            </span>
            <span className="ui-muted"> · {t("analysis.output")} </span>
            <span className="text-white">
              {(snapshot.summary.outputRatio * 100).toFixed(1)}%
            </span>
          </div>
        </div>

        <AiReportView
          expanded={reportExpanded}
          onToggleExpand={() => setReportExpanded((v) => !v)}
          loading={aiLoading}
          error={aiError}
          aiText={aiText}
          insights={snapshot.insights}
          onGenerateAi={() => void fetchAiAnalysis()}
        />
      </div>
    </aside>
  );
});
