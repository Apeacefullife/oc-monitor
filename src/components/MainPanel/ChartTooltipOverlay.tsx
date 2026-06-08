import { useLayoutEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { useT } from "../../i18n";
import { formatTokensDetailed } from "../../utils/format";

export interface ChartTooltipDay {
  date: string;
  input_cache_hit_tokens: number;
  input_cache_miss_tokens: number;
  output_tokens: number;
  cost: number;
}

export interface ChartTooltipState {
  x: number;
  y: number;
  day: ChartTooltipDay;
}

const TOKEN_COLORS = {
  cacheHit: "#60a5fa",
  cacheMiss: "#fbbf24",
  output: "#2563eb",
} as const;

const OFFSET = 14;
const MARGIN = 8;

function dayTokenTotal(day: ChartTooltipDay) {
  return (
    day.input_cache_hit_tokens +
    day.input_cache_miss_tokens +
    day.output_tokens
  );
}

function computePosition(
  anchorX: number,
  anchorY: number,
  width: number,
  height: number,
) {
  const vw = window.innerWidth;
  const vh = window.innerHeight;

  let left = anchorX + OFFSET;
  let top = anchorY + OFFSET;

  if (left + width > vw - MARGIN) {
    left = anchorX - width - OFFSET;
  }
  if (top + height > vh - MARGIN) {
    top = anchorY - height - OFFSET;
  }

  left = Math.max(MARGIN, Math.min(left, vw - width - MARGIN));
  top = Math.max(MARGIN, Math.min(top, vh - height - MARGIN));

  return { left, top };
}

function TooltipRow({
  color,
  label,
  value,
}: {
  color: string;
  label: string;
  value: number;
}) {
  return (
    <div className="flex items-center justify-between gap-6 mt-2 text-[12px] leading-snug">
      <span className="flex items-center gap-2 text-white shrink-0">
        <span
          className="w-2.5 h-2.5 rounded-[3px] shrink-0"
          style={{ background: color }}
        />
        {label}
      </span>
      <span className="text-white tabular-nums whitespace-nowrap">
        {formatTokensDetailed(value)}
      </span>
    </div>
  );
}

export default function ChartTooltipOverlay({
  state,
  useCost,
}: {
  state: ChartTooltipState | null;
  useCost: boolean;
}) {
  const t = useT();
  const panelRef = useRef<HTMLDivElement>(null);
  const [position, setPosition] = useState<{ left: number; top: number } | null>(
    null,
  );

  useLayoutEffect(() => {
    if (!state || !panelRef.current) {
      setPosition(null);
      return;
    }

    const { offsetWidth, offsetHeight } = panelRef.current;
    setPosition(
      computePosition(state.x, state.y, offsetWidth, offsetHeight),
    );
  }, [state, useCost]);

  if (!state) return null;

  const { day } = state;
  const visible = position !== null;

  return createPortal(
    <div
      className="chart-tooltip-layer"
      style={{
        left: position?.left ?? state.x + OFFSET,
        top: position?.top ?? state.y + OFFSET,
        visibility: visible ? "visible" : "hidden",
      }}
    >
      <div ref={panelRef} className="chart-tooltip-glass-panel w-max">
        {useCost ? (
          <div className="text-sm font-semibold text-white whitespace-nowrap">
            <div className="mb-1">{day.date}</div>
            <div className="tabular-nums">{day.cost.toFixed(2)}</div>
          </div>
        ) : (
          <>
            <div className="flex items-center justify-between gap-6 mb-0.5 text-[13px] font-semibold text-white whitespace-nowrap">
              <span>{day.date}</span>
              <span className="tabular-nums">
                {formatTokensDetailed(dayTokenTotal(day))}
              </span>
            </div>
            <TooltipRow
              color={TOKEN_COLORS.cacheHit}
              label={t("analysis.tooltipInputHit")}
              value={day.input_cache_hit_tokens}
            />
            <TooltipRow
              color={TOKEN_COLORS.cacheMiss}
              label={t("analysis.tooltipInputMiss")}
              value={day.input_cache_miss_tokens}
            />
            <TooltipRow
              color={TOKEN_COLORS.output}
              label={t("analysis.tooltipOutput")}
              value={day.output_tokens}
            />
          </>
        )}
      </div>
    </div>,
    document.body,
  );
}
