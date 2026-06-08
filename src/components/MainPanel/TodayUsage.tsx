import type { DailyUsage } from "../../types";
import { useT } from "../../i18n";
import { formatCurrency, formatRequestCount, formatTokens } from "../../utils/format";

interface Props {
  dailyUsage: DailyUsage[];
  currency?: string;
  isMonthlyFallback?: boolean;
}

function buildMeta(parts: Array<string | null | undefined>) {
  return parts.filter(Boolean).join(" · ");
}

export default function TodayUsage({
  dailyUsage,
  currency = "CNY",
  isMonthlyFallback,
}: Props) {
  const t = useT();
  const todayStr = new Date().toISOString().slice(0, 10);
  const today = dailyUsage.find((u) => u.date === todayStr);

  const showCost = today && today.cost >= 0;
  const cost = showCost ? formatCurrency(today.cost, currency) : "--";
  const meta = buildMeta([
    today && today.total_tokens > 0 ? formatTokens(today.total_tokens) : null,
    today && today.request_count > 0
      ? t("usage.requestCount", { n: formatRequestCount(today.request_count) })
      : null,
  ]);

  return (
    <div className="glass-card metric-card animate-fade-in h-full">
      <div className="metric-card__head">
        <span className="ui-label text-[10px] tracking-wide leading-none">
          {isMonthlyFallback ? t("usage.monthTotal") : t("usage.todayCost")}
        </span>
      </div>
      <div className="metric-card__value">
        <span className="text-[15px] font-semibold ui-value-cost font-mono leading-none truncate">
          {showCost ? cost : "--"}
        </span>
      </div>
      <div className="metric-card__meta">
        <span className="text-[10px] ui-muted font-mono leading-none truncate">
          {meta || "\u00a0"}
        </span>
      </div>
    </div>
  );
}
