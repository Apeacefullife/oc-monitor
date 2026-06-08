import { useCallback, useMemo, useRef, useState } from "react";
import ReactECharts from "echarts-for-react";
import { useT } from "../../i18n";
import type { DailyCacheRow } from "../../utils/usageAnalysis";
import { formatTokens } from "../../utils/format";
import ChartTooltipOverlay, {
  type ChartTooltipState,
} from "../MainPanel/ChartTooltipOverlay";

interface Props {
  data: DailyCacheRow[];
}

const BAR_RADIUS_TOP: [number, number, number, number] = [4, 4, 0, 0];
const CHART_GRID = { left: 2, right: 2, top: 22, bottom: 16 };

export default function CacheTokenChart({ data }: Props) {
  const t = useT();
  const chartRef = useRef<ReactECharts>(null);
  const [tooltip, setTooltip] = useState<ChartTooltipState | null>(null);

  const series = useMemo(() => data.slice(-7), [data]);
  const labels = series.map((d) => d.date.slice(5));
  const hits = series.map((d) => d.hit);
  const misses = series.map((d) => d.miss);
  const totalHit = hits.reduce((s, v) => s + v, 0);
  const totalMiss = misses.reduce((s, v) => s + v, 0);

  const syncBarHighlight = useCallback(
    (dataIndex: number | null) => {
      const inst = chartRef.current?.getEchartsInstance();
      if (!inst) return;

      inst.dispatchAction({ type: "downplay", seriesIndex: 0 });
      inst.dispatchAction({ type: "downplay", seriesIndex: 1 });

      if (dataIndex === null || dataIndex < 0 || dataIndex >= series.length) {
        return;
      }

      inst.dispatchAction({
        type: "highlight",
        seriesIndex: 0,
        dataIndex,
      });
      inst.dispatchAction({
        type: "highlight",
        seriesIndex: 1,
        dataIndex,
      });
    },
    [series.length],
  );

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

      const day = series[idx];
      syncBarHighlight(idx);
      setTooltip({
        x: clientX,
        y: clientY,
        day: {
          date: day.date,
          input_cache_hit_tokens: day.hit,
          input_cache_miss_tokens: day.miss,
          output_tokens: day.output,
          cost: day.cost,
        },
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
      yAxis: { show: false },
      series: [
        {
          name: t("analysis.cacheHit"),
          type: "bar",
          stack: "input",
          data: hits,
          barMaxWidth: 18,
          barCategoryGap: "38%",
          itemStyle: {
            color: "#60a5fa",
            borderRadius: [0, 0, 0, 0],
          },
          emphasis: {
            focus: "none",
            itemStyle: { color: "#93c5fd" },
          },
        },
        {
          name: t("analysis.cacheMiss"),
          type: "bar",
          stack: "input",
          data: misses,
          barMaxWidth: 18,
          itemStyle: {
            color: "rgba(251,191,36,0.75)",
            borderRadius: BAR_RADIUS_TOP,
          },
          emphasis: {
            focus: "none",
            itemStyle: { color: "rgba(253,224,71,0.9)" },
          },
        },
      ],
    }),
    [labels, hits, misses, t],
  );

  if (series.length === 0) return null;

  return (
    <>
      <div className="analysis-metric-card">
        <div className="flex items-center justify-between mb-1">
          <span className="analysis-section-title mb-0">{t("analysis.inputComposition")}</span>
          <span className="text-[10px] ui-muted tabular-nums">
            {formatTokens(totalHit)} / {formatTokens(totalMiss)}
          </span>
        </div>
        <div className="flex items-center gap-2 mb-1">
          <span className="flex items-center gap-1 text-[9px] ui-muted">
            <span className="w-2 h-2 rounded-sm bg-deepseek-400" />
            {t("analysis.cacheHit")}
          </span>
          <span className="flex items-center gap-1 text-[9px] ui-muted">
            <span className="w-2 h-2 rounded-sm bg-warning/75" />
            {t("analysis.cacheMiss")}
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
            style={{ height: 80, width: "100%" }}
            opts={{ renderer: "svg" }}
          />
        </div>
      </div>
      <ChartTooltipOverlay state={tooltip} useCost={false} />
    </>
  );
}
