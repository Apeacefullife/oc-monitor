import { useMemo } from "react";
import type { DailyUsage, MonthlyCost } from "../../types";
import { useAppStore } from "../../stores/useAppStore";
import { useSettingsStore } from "../../stores/useSettingsStore";
import { aggregateUsage, filterByDataSource } from "../../utils/usageAggregate";
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
  const dataSource = useSettingsStore((s) => s.dataSource);
  const rawRecords = useAppStore((s) => s.rawRecords);

  // 按当前 dataSource 重新过滤 + 聚合（瞬时切换的关键）
  const usage = useMemo(
    () => aggregateUsage(filterByDataSource(rawRecords, dataSource)),
    [rawRecords, dataSource],
  );

  // 派生 props 优先用 useMemo 结果，保留 props 兜底
  const effectiveDaily = usage.daily.length > 0 ? usage.daily : dailyUsage;
  const effectiveModels = usage.models.length > 0 ? usage.models : modelUsage;
  const effectiveCurrency = usage.monthly.currency || usageCurrency;
  const effectiveHasDaily = usage.has_daily_granularity || hasDailyGranularity;

  return (
    <>
      <BalanceCard dailyUsage={effectiveDaily} />

      <div className="grid grid-cols-2 gap-2 items-stretch">
        <TodayUsage
          dailyUsage={effectiveDaily}
          currency={effectiveCurrency}
          isMonthlyFallback={!effectiveHasDaily}
        />
        <ThisWeekUsage
          dailyUsage={effectiveDaily}
          currency={effectiveCurrency}
        />
      </div>

      <ModelUsageList models={effectiveModels} currency={effectiveCurrency} selectedModels={selectedModels} />

      <UsageTrendChart dailyUsage={effectiveDaily} />
    </>
  );
}
