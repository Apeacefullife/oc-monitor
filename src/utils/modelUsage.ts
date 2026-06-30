import type { DailyUsage } from "../types";

/** 跟踪的模型 ID 列表 */
export const TRACKED_MODEL_IDS = [
  "deepseek-v4-flash",
  "deepseek-v4-pro",
] as const;

export type TrackedModelId = (typeof TRACKED_MODEL_IDS)[number];

export function normalizeModelId(model: string): string {
  const n = model.trim().toLowerCase();
  if (!n || n === "all") return n;

  // DeepSeek
  if (n.includes("v4-pro") || n.includes("deepseek-reasoner")) return "deepseek-v4-pro";
  if (n.includes("v4-flash") || n.includes("deepseek-chat")) return "deepseek-v4-flash";

  // Claude
  if (n.includes("claude-sonnet-4-6") || n.includes("claude-sonnet-4")) return "claude-sonnet-4";
  if (n.includes("claude-opus-4")) return "claude-opus-4";
  if (n.includes("claude-haiku")) return "claude-haiku";

  // OpenAI
  if (n.includes("gpt-4o-mini")) return "gpt-4o-mini";
  if (n.includes("gpt-4o")) return "gpt-4o";
  if (n.includes("o1")) return "o1";
  if (n.includes("o3")) return "o3-mini";

  // Gemini
  if (n.includes("gemini-2.5-flash")) return "gemini-2.5-flash";
  if (n.includes("gemini-2.5-pro") || n.includes("gemini-2.5")) return "gemini-2.5-pro";
  if (n.includes("gemini-2.0") || n.includes("gemini-2")) return "gemini-2.0-flash";

  // Meta
  if (n.includes("llama-4")) return "llama-4";
  if (n.includes("llama")) return "llama-3";

  // Others
  if (n.includes("qwen")) return "qwen";
  if (n.includes("mistral") || n.includes("mixtral")) return "mistral";

  return model.trim();
}

function emptyModel(id: string): DailyUsage {
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

/** 合并用量并按跟踪模型列表排序 */
export function buildTrackedModelUsage(models: DailyUsage[]): DailyUsage[] {
  const aggregated = new Map<string, DailyUsage>();

  for (const row of models) {
    const id = normalizeModelId(row.model);
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

export function modelDisplayName(model: string): string {
  const names: Record<string, string> = {
    "deepseek-v4-flash": "DeepSeek V4 Flash",
    "deepseek-v4-pro": "DeepSeek V4 Pro",
    "claude-sonnet-4": "Claude Sonnet 4",
    "claude-opus-4": "Claude Opus 4",
    "claude-haiku": "Claude Haiku",
    "gpt-4o": "GPT-4o",
    "gpt-4o-mini": "GPT-4o Mini",
    "o1": "O1",
    "o3-mini": "O3 Mini",
    "gemini-2.5-flash": "Gemini 2.5 Flash",
    "gemini-2.5-pro": "Gemini 2.5 Pro",
    "gemini-2.0-flash": "Gemini 2.0 Flash",
    "llama-4": "Llama 4",
    "llama-3": "Llama 3",
    "qwen": "Qwen",
    "mistral": "Mistral",
  };
  return names[model] || model;
}

/** 是否为高级 Pro 模型（用于柱状图颜色区分） */
export function isProModelId(model: string): boolean {
  return ["deepseek-v4-pro", "claude-opus-4", "o1", "o3-mini"].includes(model);
}
