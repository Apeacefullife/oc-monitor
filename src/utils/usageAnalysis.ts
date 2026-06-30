import type { Locale } from "../i18n";
import { translate } from "../i18n";
import type { DailyUsage, MonthlyCost } from "../types";

export interface CacheStats {
  hit: number;
  miss: number;
  input: number;
  output: number;
  hitRate: number;
}

export interface DailyCacheRow {
  date: string;
  hit: number;
  miss: number;
  output: number;
  hitRate: number;
  cost: number;
}

export interface UsageAnalysisSnapshot {
  today: CacheStats;
  last7Days: CacheStats;
  month: CacheStats;
  dailyTrend: DailyCacheRow[];
  insights: string[];
  summary: {
    currency: string;
    monthlyCost: number | null;
    totalTokens7d: number;
    totalCost7d: number;
    avgDailyTokens: number;
    outputRatio: number;
  };
}

function resolveParts(row: DailyUsage) {
  const hit = row.input_cache_hit_tokens ?? 0;
  const miss = row.input_cache_miss_tokens ?? 0;
  const output = row.output_tokens ?? 0;

  if (hit > 0 || miss > 0) {
    return { hit, miss, output };
  }

  const input = row.input_tokens ?? 0;
  return { hit: 0, miss: input, output };
}

function aggregateByDate(dailyUsage: DailyUsage[]): DailyCacheRow[] {
  const map = new Map<string, DailyCacheRow>();

  for (const row of dailyUsage) {
    const parts = resolveParts(row);
    const prev = map.get(row.date) ?? {
      date: row.date,
      hit: 0,
      miss: 0,
      output: 0,
      hitRate: 0,
      cost: 0,
    };
    prev.hit += parts.hit;
    prev.miss += parts.miss;
    prev.output += parts.output;
    prev.cost += row.cost ?? 0;
    map.set(row.date, prev);
  }

  return [...map.values()]
    .map((d) => {
      const input = d.hit + d.miss;
      return {
        ...d,
        hitRate: input > 0 ? (d.hit / input) * 100 : 0,
      };
    })
    .filter((d) => d.hit + d.miss + d.output > 0 || d.cost > 0)
    .sort((a, b) => a.date.localeCompare(b.date));
}

function sumStats(rows: DailyCacheRow[]): CacheStats {
  const hit = rows.reduce((s, r) => s + r.hit, 0);
  const miss = rows.reduce((s, r) => s + r.miss, 0);
  const output = rows.reduce((s, r) => s + r.output, 0);
  const input = hit + miss;
  return {
    hit,
    miss,
    output,
    input,
    hitRate: input > 0 ? (hit / input) * 100 : 0,
  };
}

function formatRate(rate: number) {
  return rate >= 99.95 ? rate.toFixed(2) : rate.toFixed(1);
}

function buildInsights(
  locale: Locale,
  today: CacheStats,
  week: CacheStats,
  month: CacheStats,
  summary: UsageAnalysisSnapshot["summary"],
): string[] {
  const insights: string[] = [];
  const t = (key: string, params?: Record<string, string | number>) =>
    translate(locale, key, params);

  if (week.input > 0) {
    insights.push(
      t("analysis.insightWeekHitRate", {
        rate: formatRate(week.hitRate),
        hit: week.hit.toLocaleString(),
        miss: week.miss.toLocaleString(),
      }),
    );
  }

  if (today.input > 0) {
    if (today.hitRate >= 95) {
      insights.push(t("analysis.insightTodayHighHit"));
    } else if (today.hitRate < 70) {
      insights.push(t("analysis.insightTodayLowHit"));
    }
  }

  if (week.hitRate > month.hitRate + 5 && month.input > 0) {
    insights.push(t("analysis.insightWeekAboveMonth"));
  } else if (month.hitRate > week.hitRate + 5 && month.input > 0) {
    insights.push(t("analysis.insightWeekBelowMonth"));
  }

  if (summary.outputRatio > 0.35) {
    insights.push(
      t("analysis.insightHighOutput", {
        ratio: (summary.outputRatio * 100).toFixed(1),
      }),
    );
  }

  if (insights.length === 0) {
    insights.push(t("analysis.insightInsufficientData"));
  }

  return insights;
}

export function buildUsageAnalysisSnapshot(
  dailyUsage: DailyUsage[],
  monthlyCost: MonthlyCost | null,
  currency: string,
  locale: Locale = "zh",
): UsageAnalysisSnapshot {
  const dailyTrend = aggregateByDate(dailyUsage);
  const todayStr = new Date().toISOString().slice(0, 10);
  const monthPrefix = todayStr.slice(0, 7);

  const todayRows = dailyTrend.filter((d) => d.date === todayStr);
  const weekRows = dailyTrend.slice(-7);
  const monthRows = dailyTrend.filter((d) => d.date.startsWith(monthPrefix));

  const today = sumStats(todayRows);
  const last7Days = sumStats(weekRows);
  const month = sumStats(monthRows.length > 0 ? monthRows : dailyTrend);

  const totalTokens7d = weekRows.reduce(
    (s, d) => s + d.hit + d.miss + d.output,
    0,
  );
  const totalCost7d = weekRows.reduce((s, d) => s + d.cost, 0);
  const totalInput7d = last7Days.input;
  const outputRatio =
    totalInput7d + last7Days.output > 0
      ? last7Days.output / (totalInput7d + last7Days.output)
      : 0;

  const summary = {
    currency,
    monthlyCost: monthlyCost?.total_cost ?? null,
    totalTokens7d,
    totalCost7d,
    avgDailyTokens: weekRows.length > 0 ? totalTokens7d / weekRows.length : 0,
    outputRatio,
  };

  return {
    today,
    last7Days,
    month,
    dailyTrend: weekRows,
    insights: buildInsights(locale, today, last7Days, month, summary),
    summary,
  };
}

export function formatHitRate(rate: number) {
  return `${formatRate(rate)}%`;
}
