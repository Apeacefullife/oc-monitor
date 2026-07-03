import type { DailyUsage, MonthlyCost } from "../../types";
import { useSettingsStore } from "../../stores/useSettingsStore";
import BalanceCard from "./BalanceCard";
import TodayUsage from "./TodayUsage";
import ThisWeekUsage from "./ThisWeekUsage";
import ModelUsageList from "./ModelUsageList";
import UsageTrendChart from "./UsageTrendChart";

interface Props {
  dailyUsage: DailyUsage[];
  modelUsage: DailyUsage[];
  monthlyCost: MonthlyCost | null;
  loading: boolean;
  usageCurrency?: string;
  hasDailyGranularity?: boolean;
}

export default function MainPanel({
  dailyUsage,
  modelUsage,
  usageCurrency = "USD",
  hasDailyGranularity = true,
}: Props) {
  const selectedModels = useSettingsStore((s) => s.selectedModels);

  return (
    <>
      <BalanceCard dailyUsage={dailyUsage} />

      <div className="grid grid-cols-2 gap-2 items-stretch">
        <TodayUsage
          dailyUsage={dailyUsage}
          currency={usageCurrency}
          isMonthlyFallback={!hasDailyGranularity}
        />
        <ThisWeekUsage
          dailyUsage={dailyUsage}
          currency={usageCurrency}
        />
      </div>

      <ModelUsageList models={modelUsage} currency={usageCurrency} selectedModels={selectedModels} />

      <UsageTrendChart dailyUsage={dailyUsage} />
    </>
  );
}
