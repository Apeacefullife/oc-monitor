import type { BalanceInfo, DailyUsage, MonthlyCost } from "../../types";
import BalanceCard from "./BalanceCard";
import TodayUsage from "./TodayUsage";
import MonthlyCostCard from "./MonthlyCost";
import ModelUsageList from "./ModelUsageList";
import UsageTrendChart from "./UsageTrendChart";

interface Props {
  balance: BalanceInfo | null;
  dailyUsage: DailyUsage[];
  modelUsage: DailyUsage[];
  monthlyCost: MonthlyCost | null;
  loading: boolean;
  usageCurrency?: string;
  hasDailyGranularity?: boolean;
}

export default function MainPanel({
  balance,
  dailyUsage,
  modelUsage,
  monthlyCost,
  usageCurrency = "CNY",
  hasDailyGranularity = true,
}: Props) {
  return (
    <>
      <BalanceCard balance={balance} dailyUsage={dailyUsage} />

      <div className="grid grid-cols-2 gap-2 items-stretch">
        <TodayUsage
          dailyUsage={dailyUsage}
          currency={usageCurrency}
          isMonthlyFallback={!hasDailyGranularity}
        />
        <MonthlyCostCard
          monthlyCost={monthlyCost}
          currency={usageCurrency}
        />
      </div>

      <ModelUsageList models={modelUsage} currency={usageCurrency} />

      <UsageTrendChart dailyUsage={dailyUsage} />
    </>
  );
}
