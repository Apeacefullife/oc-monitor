import type { MonthlyCost } from "../../types";
import { useT } from "../../i18n";
import { formatCurrency, formatRequestCount } from "../../utils/format";

interface Props {
  monthlyCost: MonthlyCost | null;
  currency?: string;
}

function monthLabel(month: string | undefined, t: ReturnType<typeof useT>) {
  if (!month) return null;
  const parts = month.split("-");
  if (parts.length === 2) {
    return t("usage.monthLabel", { n: Number(parts[1]) });
  }
  return month;
}

export default function MonthlyCostCard({
  monthlyCost,
  currency = "CNY",
}: Props) {
  const t = useT();
  const cur = monthlyCost?.currency ?? currency;
  const hasData = monthlyCost && monthlyCost.total_cost > 0;
  const requests =
    monthlyCost && (monthlyCost.request_count ?? 0) > 0
      ? t("usage.requestCount", {
          n: formatRequestCount(monthlyCost.request_count!),
        })
      : null;
  const month = monthLabel(monthlyCost?.month, t);

  return (
    <div className="glass-card metric-card animate-fade-in h-full">
      <div className="metric-card__head">
        <span className="ui-label text-[10px] tracking-wide leading-none">
          {t("usage.monthCost")}
        </span>
        {month && (
          <span className="text-[10px] ui-muted font-mono leading-none shrink-0">
            {month}
          </span>
        )}
      </div>
      <div className="metric-card__value">
        <span className="text-[15px] font-semibold ui-value-cost font-mono leading-none truncate">
          {hasData ? formatCurrency(monthlyCost.total_cost, cur) : "--"}
        </span>
      </div>
      <div className="metric-card__meta">
        <span className="text-[10px] ui-muted font-mono leading-none truncate">
          {requests || "\u00a0"}
        </span>
      </div>
    </div>
  );
}
