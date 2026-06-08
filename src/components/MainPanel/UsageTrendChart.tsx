import { useCallback, useRef, useState } from "react";
import ReactECharts from "echarts-for-react";
import { useT } from "../../i18n";
import type { DailyUsage } from "../../types";
import { formatTokens } from "../../utils/format";
import ChartTooltipOverlay, {
  type ChartTooltipState,
} from "./ChartTooltipOverlay";

interface Props {
  dailyUsage: DailyUsage[];
}

interface DayAggregate {
  date: string;
  total_tokens: number;
  input_cache_hit_tokens: number;
  input_cache_miss_tokens: number;
  output_tokens: number;
  cost: number;
}

const BAR_COLOR = "#7eb8fc";
const BAR_COLOR_EMPHASIS = "#bae6fd";
const COST_BAR_COLOR = "#fde047";
const COST_BAR_EMPHASIS = "#fef9c3";
const BAR_RADIUS: [number, number, number, number] = [6, 6, 3, 3];

const CHART_GRID = { left: 8, right: 8, top: 30, bottom: 22 };

const CHART_X_AXIS = (labels: string[]) => ({
  type: "category" as const,
  data: labels,
  axisLine: { show: false },
  axisTick: { show: false },
  axisLabel: {
    color: "rgba(255,255,255,0.78)",
    fontSize: 10,
    fontWeight: 500,
    margin: 8,
  },
});

function resolveTokenParts(row: DailyUsage) {
  const hit = row.input_cache_hit_tokens ?? 0;
  const miss = row.input_cache_miss_tokens ?? 0;
  const output = row.output_tokens ?? 0;

  if (hit > 0 || miss > 0) {
    return { hit, miss, output };
  }

  const input = row.input_tokens ?? 0;
  return { hit: 0, miss: input, output };
}

function aggregateByDate(dailyUsage: DailyUsage[]): DayAggregate[] {
  const map = new Map<string, DayAggregate>();
  for (const row of dailyUsage) {
    const parts = resolveTokenParts(row);
    const prev = map.get(row.date) ?? {
      date: row.date,
      total_tokens: 0,
      input_cache_hit_tokens: 0,
      input_cache_miss_tokens: 0,
      output_tokens: 0,
      cost: 0,
    };
    prev.input_cache_hit_tokens += parts.hit;
    prev.input_cache_miss_tokens += parts.miss;
    prev.output_tokens += parts.output;
    prev.total_tokens +=
      row.total_tokens || parts.hit + parts.miss + parts.output;
    prev.cost += row.cost ?? 0;
    map.set(row.date, prev);
  }
  return [...map.values()]
    .filter(
      (d) =>
        d.total_tokens > 0 ||
        d.cost > 0 ||
        d.input_cache_hit_tokens > 0 ||
        d.input_cache_miss_tokens > 0 ||
        d.output_tokens > 0,
    )
    .sort((a, b) => a.date.localeCompare(b.date));
}

function dayTokenTotal(day: DayAggregate) {
  return (
    day.input_cache_hit_tokens +
    day.input_cache_miss_tokens +
    day.output_tokens
  );
}

export default function UsageTrendChart({ dailyUsage }: Props) {
  const t = useT();
  const chartRef = useRef<ReactECharts>(null);
  const [tooltip, setTooltip] = useState<ChartTooltipState | null>(null);

  const aggregated = aggregateByDate(dailyUsage);
  const useCost =
    aggregated.every((d) => dayTokenTotal(d) <= 0) &&
    aggregated.some((d) => d.cost > 0);
  const series = aggregated.slice(-7);
  const total = series.reduce(
    (s, d) => s + (useCost ? d.cost : dayTokenTotal(d)),
    0,
  );

  const syncBarHighlight = useCallback((dataIndex: number | null) => {
    const inst = chartRef.current?.getEchartsInstance();
    if (!inst) return;

    inst.dispatchAction({ type: "downplay", seriesIndex: 0 });

    if (dataIndex === null || dataIndex < 0 || dataIndex >= series.length) {
      return;
    }

    inst.dispatchAction({
      type: "highlight",
      seriesIndex: 0,
      dataIndex,
    });
  }, [series.length]);

  const updateTooltipFromMouse = useCallback(
    (clientX: number, clientY: number) => {
      const inst = chartRef.current?.getEchartsInstance();
      if (!inst) return;

      const dom = inst.getDom();
      const rect = dom.getBoundingClientRect();
      const offsetX = clientX - rect.left;
      const offsetY = clientY - rect.top;

      const dataCoord = inst.convertFromPixel({ seriesIndex: 0 }, [
        offsetX,
        offsetY,
      ]);
      const idx = Math.round(dataCoord[0]);

      if (idx < 0 || idx >= series.length || Number.isNaN(idx)) {
        setTooltip(null);
        syncBarHighlight(null);
        return;
      }

      syncBarHighlight(idx);
      setTooltip({
        x: clientX,
        y: clientY,
        day: series[idx],
      });
    },
    [series, syncBarHighlight],
  );

  const handleChartMouseMove = useCallback(
    (e: React.MouseEvent<HTMLDivElement>) => {
      updateTooltipFromMouse(e.clientX, e.clientY);
    },
    [updateTooltipFromMouse],
  );

  const handleChartLeave = useCallback(() => {
    setTooltip(null);
    syncBarHighlight(null);
  }, [syncBarHighlight]);

  if (series.length === 0) {
    return (
      <div className="glass-card p-4 animate-fade-in">
        <div className="ui-label text-xs tracking-wide">{t("usage.usageTrend")}</div>
        <div className="ui-muted text-xs mt-2">{t("usage.noData")}</div>
      </div>
    );
  }

  const xLabels = series.map((d) => d.date.slice(5));

  const option = useCost
    ? {
        grid: CHART_GRID,
        tooltip: { show: false },
        xAxis: CHART_X_AXIS(xLabels),
        yAxis: { show: false },
        series: [
          {
            type: "bar",
            data: series.map((d) => d.cost),
            barMaxWidth: 24,
            barCategoryGap: "42%",
            itemStyle: {
              borderRadius: BAR_RADIUS,
              color: COST_BAR_COLOR,
            },
            emphasis: {
              focus: "none",
              itemStyle: {
                color: COST_BAR_EMPHASIS,
                borderRadius: BAR_RADIUS,
              },
            },
            label: {
              show: true,
              position: "top",
              distance: 4,
              formatter: (p: { value: number }) =>
                p.value <= 0 ? "0" : p.value.toFixed(2),
              color: "rgba(255,255,255,0.9)",
              fontSize: 9,
              fontWeight: 600,
            },
          },
        ],
      }
    : {
        grid: CHART_GRID,
        tooltip: { show: false },
        xAxis: CHART_X_AXIS(xLabels),
        yAxis: { show: false },
        series: [
          {
            type: "bar",
            data: series.map((d) => dayTokenTotal(d)),
            barMaxWidth: 24,
            barCategoryGap: "42%",
            itemStyle: {
              borderRadius: BAR_RADIUS,
              color: BAR_COLOR,
            },
            emphasis: {
              focus: "none",
              itemStyle: {
                color: BAR_COLOR_EMPHASIS,
                borderRadius: BAR_RADIUS,
              },
            },
            label: {
              show: true,
              position: "top",
              distance: 4,
              formatter: (p: { dataIndex: number }) => {
                const day = series[p.dataIndex];
                const value = day ? dayTokenTotal(day) : 0;
                if (value <= 0) return "0";
                return formatTokens(value);
              },
              color: "rgba(255,255,255,0.9)",
              fontSize: 9,
              fontWeight: 600,
            },
          },
        ],
      };

  return (
    <>
      <div className="glass-card p-4 animate-fade-in">
        <div className="flex items-center justify-between mb-2">
          <span className="ui-label text-xs tracking-wide">{t("usage.usageTrend")}</span>
          <span className="text-[11px] ui-muted font-mono font-medium">
            {useCost ? total.toFixed(2) : formatTokens(total)}
          </span>
        </div>
        <div
          className="ds-chart-surface"
          onMouseMove={handleChartMouseMove}
          onMouseLeave={handleChartLeave}
        >
          <ReactECharts
            ref={chartRef}
            className="ds-chart"
            option={option}
            style={{ height: 112, width: "100%" }}
            opts={{ renderer: "svg" }}
          />
        </div>
      </div>
      <ChartTooltipOverlay state={tooltip} useCost={useCost} />
    </>
  );
}
