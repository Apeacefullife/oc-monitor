import type { DailyUsage } from "../../types";
import { useT } from "../../i18n";
import { formatCurrency, formatTokens } from "../../utils/format";
import {
  buildTrackedModelUsage,
  modelDisplayName,
  isProModelId,
} from "../../utils/modelUsage";

interface Props {
  models: DailyUsage[];
  currency?: string;
}

const MIN_BAR_WIDTH = 4;

function sumTokens(rows: DailyUsage[]): number {
  return rows.reduce((sum, row) => sum + row.total_tokens, 0);
}

function barFillClass(model: string) {
  return isProModelId(model) ? "ui-bar-violet" : "ui-bar-blue";
}

function renderUsageBar(
  model: DailyUsage,
  modelTotalTokens: number,
) {
  if (model.total_tokens <= 0 || modelTotalTokens <= 0) {
    return <div className="metric-card__bar ui-bar-track w-full opacity-35" aria-hidden />;
  }

  const share = Math.max(
    MIN_BAR_WIDTH,
    (model.total_tokens / modelTotalTokens) * 100,
  );

  return (
    <div className="metric-card__bar ui-bar-track w-full">
      <div
        className={`h-full rounded-full ${barFillClass(model.model)}`}
        style={{ width: `${share}%` }}
      />
    </div>
  );
}

export default function ModelUsageList({
  models,
  currency = "CNY",
}: Props) {
  const t = useT();
  const rows = buildTrackedModelUsage(models);
  const modelTotalTokens = sumTokens(rows.filter((m) => m.total_tokens > 0));
  const hasAnyUsage = modelTotalTokens > 0;

  return (
    <div className="space-y-2">
      {rows.map((model) => {
        const efficiency =
          model.cost > 0
            ? `${formatTokens(model.total_tokens / model.cost)} T/${currency === "USD" ? "$" : "¥"}`
            : "--";

        const tokenLabel = hasAnyUsage
          ? formatTokens(model.total_tokens)
          : t("usage.noData");

        return (
          <div key={model.model} className="glass-card metric-card animate-fade-in">
            <div className="metric-card__head">
              <span className="text-[11px] font-semibold text-white leading-none truncate">
                {modelDisplayName(model.model)}
              </span>
              <span
                className={`text-[10px] font-mono font-medium leading-none shrink-0 ${
                  model.total_tokens > 0 ? "ui-muted" : "ui-muted opacity-60"
                }`}
              >
                {model.total_tokens > 0 ? tokenLabel : hasAnyUsage ? "0" : tokenLabel}
              </span>
            </div>
            <div className="metric-card__value">
              {renderUsageBar(model, modelTotalTokens)}
            </div>
            <div className="metric-card__meta justify-between gap-2">
              <span className="ui-value-cost font-mono font-semibold text-[10px] leading-none truncate">
                {model.cost > 0
                  ? formatCurrency(model.cost, currency)
                  : hasAnyUsage
                    ? formatCurrency(0, currency)
                    : "--"}
              </span>
              <span className="ui-muted font-mono text-[10px] leading-none shrink-0">
                {efficiency}
              </span>
            </div>
          </div>
        );
      })}
    </div>
  );
}
