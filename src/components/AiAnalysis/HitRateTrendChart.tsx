import { useMemo } from "react";
import ReactECharts from "echarts-for-react";
import { useT } from "../../i18n";
import type { DailyCacheRow } from "../../utils/usageAnalysis";
import { formatHitRate } from "../../utils/usageAnalysis";

interface Props {
  data: DailyCacheRow[];
}

const CHART_GRID = { left: 2, right: 2, top: 22, bottom: 16 };

export default function HitRateTrendChart({ data }: Props) {
  const t = useT();
  const series = useMemo(() => data.slice(-7), [data]);
  const labels = series.map((d) => d.date.slice(5));
  const rates = series.map((d) => Number(d.hitRate.toFixed(2)));
  const avg =
    rates.length > 0 ? rates.reduce((s, v) => s + v, 0) / rates.length : 0;
  const yMin =
    rates.length > 0 ? Math.max(0, Math.floor(Math.min(...rates) - 3)) : 0;

  const option = useMemo(
    () => ({
      grid: CHART_GRID,
      tooltip: { show: false },
      xAxis: {
        type: "category" as const,
        data: labels,
        axisLine: { show: false },
        axisTick: { show: false },
        axisLabel: {
          color: "rgba(255,255,255,0.65)",
          fontSize: 9,
          margin: 6,
        },
      },
      yAxis: {
        show: false,
        min: yMin,
        max: 100,
      },
      series: [
        {
          type: "line",
          data: rates,
          smooth: 0.35,
          symbol: "circle",
          symbolSize: 5,
          lineStyle: { color: "#60a5fa", width: 2 },
          itemStyle: { color: "#93c5fd", borderColor: "#60a5fa", borderWidth: 1 },
          areaStyle: {
            color: {
              type: "linear",
              x: 0,
              y: 0,
              x2: 0,
              y2: 1,
              colorStops: [
                { offset: 0, color: "rgba(96,165,250,0.28)" },
                { offset: 1, color: "rgba(96,165,250,0.02)" },
              ],
            },
          },
          label: {
            show: true,
            position: "top",
            distance: 2,
            formatter: (p: { value: number }) =>
              p.value > 0 ? `${p.value.toFixed(1)}%` : "",
            color: "rgba(255,255,255,0.85)",
            fontSize: 8,
            fontWeight: 600,
          },
          markLine: {
            silent: true,
            symbol: "none",
            lineStyle: { color: "rgba(251,191,36,0.45)", type: "dashed", width: 1 },
            label: { show: false },
            data: [{ yAxis: avg }],
          },
        },
      ],
    }),
    [labels, rates, avg, yMin],
  );

  if (series.length === 0) return null;

  return (
    <div className="analysis-metric-card">
      <div className="flex items-center justify-between mb-1">
        <span className="analysis-section-title mb-0">{t("analysis.hitRateTrend")}</span>
        <span className="text-[10px] text-warning/80 tabular-nums">
          {t("analysis.avg")} {formatHitRate(avg)}
        </span>
      </div>
      <div className="ds-chart-surface">
        <ReactECharts
          className="ds-chart"
          option={option}
          style={{ height: 88, width: "100%" }}
          opts={{ renderer: "svg" }}
        />
      </div>
    </div>
  );
}
