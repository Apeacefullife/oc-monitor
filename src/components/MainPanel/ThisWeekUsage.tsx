import type { DailyUsage } from "../../types";
import { useT } from "../../i18n";
import { formatCurrency, formatRequestCount, formatTokens } from "../../utils/format";

interface Props {
  dailyUsage: DailyUsage[];
  currency?: string;
}

function getWeekStart(): string {
  const now = new Date();
  const day = now.getDay();
  const diff = day === 0 ? 6 : day - 1;
  const monday = new Date(now);
  monday.setDate(now.getDate() - diff);
  return monday.toISOString().slice(0, 10);
}

function getTodayStr(): string {
  return new Date().toISOString().slice(0, 10);
}

export default function ThisWeekUsage({
  dailyUsage,
  currency = "CNY",
}: Props) {
  const t = useT();
  const weekStart = getWeekStart();
  const todayStr = getTodayStr();

  const weekEntries = dailyUsage.filter(
    (u) => u.date >= weekStart && u.date <= todayStr,
  );
  const weekCost = weekEntries.reduce((s, u) => s + u.cost, 0);
  const weekTokens = weekEntries.reduce((s, u) => s + u.total_tokens, 0);
  const weekRequests = weekEntries.reduce((s, u) => s + u.request_count, 0);

  const hasData = weekCost > 0 || weekTokens > 0;

  return (
    <div className="glass-card metric-card animate-fade-in h-full">
      <div className="metric-card__head">
        <span className="ui-label text-[10px] tracking-wide leading-none">
          {t("usage.weekCost")}
        </span>
      </div>
      <div className="metric-card__value">
        <span className="text-[15px] font-semibold ui-value-cost font-mono leading-none truncate">
          {hasData ? formatCurrency(weekCost, currency) : "--"}
        </span>
      </div>
      <div className="metric-card__meta">
        <span className="text-[10px] ui-muted font-mono leading-none truncate">
          {hasData
            ? `${formatTokens(weekTokens)} · ${t("usage.requestCount", { n: formatRequestCount(weekRequests) })}`
            : "\u00a0"}
        </span>
      </div>
    </div>
  );
}
