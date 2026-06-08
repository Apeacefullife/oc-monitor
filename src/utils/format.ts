import type { Locale } from "../i18n";
import { getLocale, translate } from "../i18n";

function localeTag(locale: Locale = getLocale()) {
  return locale === "zh" ? "zh-CN" : "en-US";
}

/** 格式化请求次数 */
export function formatRequestCount(
  value: number,
  locale: Locale = getLocale(),
): string {
  return value.toLocaleString(localeTag(locale));
}

/** 格式化金额 */
export function formatCurrency(
  value: number,
  currency = "USD",
  locale: Locale = getLocale(),
): string {
  return new Intl.NumberFormat(localeTag(locale), {
    style: "currency",
    currency,
    minimumFractionDigits: 2,
    maximumFractionDigits: 4,
  }).format(value);
}

/** 格式化 Token 数量（带千分位，用于 tooltip） */
export function formatTokensDetailed(
  value: number,
  locale: Locale = getLocale(),
): string {
  const suffix = locale === "zh" ? " tokens" : " tokens";
  return `${value.toLocaleString(localeTag(locale))}${suffix}`;
}

/** 格式化 Token 数量 */
export function formatTokens(value: number, locale: Locale = getLocale()): string {
  if (value >= 1_000_000) {
    return `${(value / 1_000_000).toFixed(1)}M`;
  }
  if (value >= 1_000) {
    return `${(value / 1_000).toFixed(1)}K`;
  }
  return value.toLocaleString(localeTag(locale));
}

/** 格式化日期 */
export function formatDate(dateStr: string, locale: Locale = getLocale()): string {
  const date = new Date(dateStr);
  return new Intl.DateTimeFormat(localeTag(locale), {
    month: "short",
    day: "numeric",
  }).format(date);
}

/** 格式化时间戳为相对时间 */
export function formatRelativeTime(
  dateStr: string,
  locale: Locale = getLocale(),
): string {
  const now = Date.now();
  const then = new Date(dateStr).getTime();
  const diffMs = now - then;

  const seconds = Math.floor(diffMs / 1000);
  if (seconds < 60) return translate(locale, "status.justNow");
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) {
    return translate(locale, "status.minutesAgo", { n: minutes });
  }
  const hours = Math.floor(minutes / 60);
  if (hours < 24) {
    return translate(locale, "status.hoursAgo", { n: hours });
  }
  const days = Math.floor(hours / 24);
  return translate(locale, "status.daysAgo", { n: days });
}

/** 获取余额状态文本 */
export function getBalanceStatusText(
  status: string,
  locale: Locale = getLocale(),
): string {
  switch (status) {
    case "normal":
      return translate(locale, "balance.normal");
    case "low":
      return translate(locale, "balance.low");
    case "exhausted":
      return translate(locale, "balance.exhausted");
    default:
      return translate(locale, "balance.unknown");
  }
}

/** 获取余额状态颜色 */
export function getBalanceStatusColor(status: string): string {
  switch (status) {
    case "normal":
      return "text-success";
    case "low":
      return "text-warning";
    case "exhausted":
      return "text-danger";
    default:
      return "text-gray-400";
  }
}
