export type CursorVariant = "arrow" | "hand";

const HAND_SELECTOR = [
  "button:not(:disabled)",
  "a[href]",
  ".toggle-switch:not(.toggle-switch--disabled)",
  ".settings-ui-link",
  ".settings-ui-lang-btn",
  ".settings-ui-segment-btn",
  ".settings-ui-danger",
  ".settings-ui-close",
  ".analysis-report-toggle",
  ".analysis-report-generate",
  ".quick-action-menu-item",
  "[data-cursor-hand]",
].join(", ");

export function resolveCursorVariant(target: EventTarget | null): CursorVariant {
  if (!(target instanceof Element)) return "arrow";
  if (target.closest(HAND_SELECTOR)) return "hand";
  return "arrow";
}
