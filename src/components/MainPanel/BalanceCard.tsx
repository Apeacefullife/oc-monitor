import type { BalanceInfo, DailyUsage } from "../../types";
import { useT } from "../../i18n";
import { estimateBalanceRemainingDays } from "../../utils/balanceEstimate";
import { formatCurrency } from "../../utils/format";

interface Props {
  balance: BalanceInfo | null;
  dailyUsage: DailyUsage[];
}

export default function BalanceCard({ balance, dailyUsage }: Props) {
  const t = useT();

  const currency = balance?.currency ?? "USD";
  const available =
    balance &&
    balance.is_available !== false &&
    balance.status !== "exhausted";

  let meta: string | null = null;
  let isLowBalance = false;

  if (balance && balance.total_balance > 0) {
    const days = estimateBalanceRemainingDays(
      balance.total_balance,
      dailyUsage,
    );

    isLowBalance =
      balance.status === "low" ||
      balance.status === "exhausted" ||
      (days !== null && days < 3);

    if (days !== null) {
      meta =
        days < 1
          ? t("balance.estimatedLessThanDay")
          : t("balance.estimatedDays", { n: Math.round(days) });
    } else if (balance.status === "low") {
      meta = t("balance.low");
    }
  } else if (balance?.status === "exhausted") {
    isLowBalance = true;
  }

  return (
    <div
      className={`glass-card balance-metric-card animate-fade-in h-full${
        isLowBalance ? " balance-metric-card--low" : ""
      }`}
    >
      <div className="metric-card__head">
        <span className="ui-label text-[10px] tracking-wide leading-none">
          {t("balance.title")}
        </span>
        {balance && (
          <span
            className={`text-[10px] font-medium px-1.5 py-0.5 rounded-full leading-none shrink-0 ${
              available
                ? "bg-success/10 text-success"
                : "bg-danger/10 text-danger"
            }`}
          >
            {available ? t("balance.available") : t("balance.unavailable")}
          </span>
        )}
      </div>
      <div className="balance-metric-card__body">
        <span
          className={`balance-metric-card__amount font-mono leading-none truncate${
            isLowBalance
              ? " balance-metric-card__amount--low"
              : " ui-value-balance"
          }`}
        >
          {balance ? formatCurrency(balance.total_balance, currency) : "--"}
        </span>
        {meta && (
          <span
            className={`balance-metric-card__aside font-mono leading-snug${
              isLowBalance ? " balance-metric-card__aside--low" : " ui-muted"
            }`}
          >
            {meta}
          </span>
        )}
      </div>
    </div>
  );
}
