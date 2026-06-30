import type { DailyUsage } from "../../types";
import { useT } from "../../i18n";
import { formatCurrency, formatTokens } from "../../utils/format";

interface Props {
  dailyUsage: DailyUsage[];
}

export default function BalanceCard({ dailyUsage }: Props) {
  const t = useT();

  const monthStr = new Date().toISOString().slice(0, 7);
  const monthUsage = dailyUsage.filter((u) => u.date.startsWith(monthStr));
  const monthTokens = monthUsage.reduce((s, u) => s + u.total_tokens, 0);
  const monthCost = monthUsage.reduce((s, u) => s + u.cost, 0);

  const totalSessions = dailyUsage.reduce((s, u) => s + u.request_count, 0);

  return (
    <div className="glass-card balance-metric-card animate-fade-in h-full">
      <div className="metric-card__head">
        <span className="ui-label text-[10px] tracking-wide leading-none">
          {t("usage.monthCost")}
        </span>
      </div>
      <div className="balance-metric-card__body">
        <span className="balance-metric-card__amount font-mono leading-none truncate ui-value-balance">
          {formatCurrency(monthCost, "USD")}
        </span>
        <span className="balance-metric-card__aside font-mono leading-snug ui-muted">
          {formatTokens(monthTokens)} · {totalSessions}{t("usage.requestCount", { n: "" }).trim()}
        </span>
      </div>
    </div>
  );
}
