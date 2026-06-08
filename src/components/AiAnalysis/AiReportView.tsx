import { useMemo } from "react";
import { useT } from "../../i18n";
import {
  insightsToSections,
  mergeReportSections,
  parseStructuredReport,
  type ReportSection,
} from "../../utils/aiReport";
import LoadingSpinner from "../common/LoadingSpinner";

interface Props {
  expanded: boolean;
  onToggleExpand: () => void;
  loading: boolean;
  error: string | null;
  aiText: string | null;
  insights: string[];
  onGenerateAi: () => void;
}

function ReportBody({ sections }: { sections: ReportSection[] }) {
  return (
    <div className="analysis-report-body">
      {sections.map((section, si) => (
        <section key={`${section.title}-${si}`} className="analysis-report-section">
          {section.title && (
            <h3 className="analysis-report-section-title">{section.title}</h3>
          )}
          <ul className="analysis-report-list">
            {section.items.map((item, ii) => (
              <li key={ii} className="analysis-report-item">
                <span className="analysis-report-item-dot" aria-hidden />
                <span>{item}</span>
              </li>
            ))}
          </ul>
        </section>
      ))}
    </div>
  );
}

export default function AiReportView({
  expanded,
  onToggleExpand,
  loading,
  error,
  aiText,
  insights,
  onGenerateAi,
}: Props) {
  const t = useT();

  const sections = useMemo(() => {
    if (aiText) {
      const parsed = parseStructuredReport(aiText);
      if (parsed.length > 0) return mergeReportSections(parsed);
    }
    return insightsToSections(insights, t("analysis.localInsights"));
  }, [aiText, insights, t]);

  const hasContent = sections.length > 0;
  const canExpand = hasContent && !loading && !error;

  return (
    <div
      className={`analysis-report-panel ${
        expanded ? "analysis-report-panel--expanded" : "analysis-report-panel--collapsed"
      }`}
    >
      <div className="analysis-report-header">
        <span className="analysis-report-title">{t("analysis.reportTitle")}</span>
        <div className="analysis-report-header-actions">
          {canExpand && (
            <button
              type="button"
              onClick={onToggleExpand}
              className="analysis-report-toggle"
              aria-expanded={expanded}
            >
              {expanded ? t("analysis.collapseReport") : t("analysis.expandReport")}
              <svg
                className={`analysis-report-toggle-icon ${
                  expanded ? "" : "analysis-report-toggle-icon--up"
                }`}
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
                strokeWidth={2}
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  d="M19 9l-7 7-7-7"
                />
              </svg>
            </button>
          )}
        </div>
      </div>

      <div className="analysis-report-content">
        {loading ? (
          <div className="analysis-report-loading">
            <LoadingSpinner />
          </div>
        ) : error ? (
          <p className="analysis-report-error">{error}</p>
        ) : hasContent ? (
          <ReportBody sections={sections} />
        ) : (
          <p className="analysis-report-empty">{t("analysis.reportEmpty")}</p>
        )}
      </div>

      {!loading && !aiText && !error && (
        <button
          type="button"
          onClick={onGenerateAi}
          className="analysis-report-generate"
        >
          {t("analysis.generateAi")}
        </button>
      )}
    </div>
  );
}
