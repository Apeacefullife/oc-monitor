import type { DailyUsage } from "../types";

/** 主界面固定展示的模型（按 DeepSeek 平台 canonical id） */
export const TRACKED_MODEL_IDS = [
  "deepseek-v4-flash",
  "deepseek-v4-pro",
] as const;

export type TrackedModelId = (typeof TRACKED_MODEL_IDS)[number];

export function normalizeModelId(model: string): string {
  const n = model.trim().toLowerCase();
  if (!n || n === "all") return n;
  if (
    n.includes("v4-pro") ||
    n.includes("deepseek-reasoner") ||
    n === "deepseek-pro" ||
    n.includes("reasoner") ||
    (n.includes("pro") && !n.includes("prompt"))
  ) {
    return "deepseek-v4-pro";
  }
  if (
    n.includes("v4-flash") ||
    n.includes("deepseek-chat") ||
    n === "deepseek-flash" ||
    n.includes("chat") ||
    n.includes("flash")
  ) {
    return "deepseek-v4-flash";
  }
  return model.trim();
}

function emptyModel(id: TrackedModelId): DailyUsage {
  return {
    date: "",
    total_tokens: 0,
    input_tokens: 0,
    input_cache_hit_tokens: 0,
    input_cache_miss_tokens: 0,
    output_tokens: 0,
    request_count: 0,
    cost: 0,
    model: id,
  };
}

function mergeUsage(a: DailyUsage, b: DailyUsage): DailyUsage {
  return {
    date: a.date || b.date,
    model: a.model,
    total_tokens: a.total_tokens + b.total_tokens,
    input_tokens: a.input_tokens + b.input_tokens,
    input_cache_hit_tokens:
      (a.input_cache_hit_tokens ?? 0) + (b.input_cache_hit_tokens ?? 0),
    input_cache_miss_tokens:
      (a.input_cache_miss_tokens ?? 0) + (b.input_cache_miss_tokens ?? 0),
    output_tokens: a.output_tokens + b.output_tokens,
    request_count: a.request_count + b.request_count,
    cost: a.cost + b.cost,
  };
}

/** 合并 API 数据并补齐 Flash / Pro 两行 */
export function buildTrackedModelUsage(models: DailyUsage[]): DailyUsage[] {
  const aggregated = new Map<string, DailyUsage>();

  for (const row of models) {
    const id = normalizeModelId(row.model);
    if (!TRACKED_MODEL_IDS.includes(id as TrackedModelId)) continue;
    const prev = aggregated.get(id);
    aggregated.set(
      id,
      prev ? mergeUsage(prev, { ...row, model: id }) : { ...row, model: id },
    );
  }

  return TRACKED_MODEL_IDS.map(
    (id) => aggregated.get(id) ?? emptyModel(id),
  );
}

export function isProModelId(model: string): boolean {
  return normalizeModelId(model) === "deepseek-v4-pro";
}

export function modelDisplayName(model: string): string {
  return isProModelId(model) ? "V4 Pro" : "V4 Flash";
}
