import type { DailyUsage } from "../types";

const LOOKBACK_DAYS = 7;

/** 按近 N 日日均消耗估算余额可用天数；无消耗数据时返回 null */
export function estimateBalanceRemainingDays(
  balance: number,
  dailyUsage: DailyUsage[],
  lookbackDays = LOOKBACK_DAYS,
): number | null {
  if (balance <= 0) return null;

  const costByDate = new Map<string, number>();
  for (const row of dailyUsage) {
    costByDate.set(
      row.date,
      (costByDate.get(row.date) ?? 0) + (row.cost ?? 0),
    );
  }

  const today = new Date();
  let totalCost = 0;

  for (let i = 0; i < lookbackDays; i++) {
    const d = new Date(today);
    d.setDate(d.getDate() - i);
    const dateStr = d.toISOString().slice(0, 10);
    totalCost += costByDate.get(dateStr) ?? 0;
  }

  if (totalCost <= 0) return null;

  const avgDailyCost = totalCost / lookbackDays;
  if (avgDailyCost <= 0) return null;

  return balance / avgDailyCost;
}
