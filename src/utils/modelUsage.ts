import type { DailyUsage } from "../types";

/** 跟踪的模型 ID 列表 */
export const TRACKED_MODEL_IDS = [
  "mimo-v2.5",
  "mimo-v2.5-pro",
  "deepseek-v4-flash",
  "deepseek-v4-pro",
  "glm-5.1",
  "glm-5.2",
  "kimi-k2.6",
  "kimi-k2.7-code",
  "minimax-m2.5",
  "minimax-m2.7",
  "minimax-m3",
  "qwen3.6-plus-le256k",
  "qwen3.6-plus-gt256k",
  "qwen3.7-max",
  "qwen3.7-plus-le256k",
  "qwen3.7-plus-gt256k",
] as const;

export type TrackedModelId = (typeof TRACKED_MODEL_IDS)[number];

export function normalizeModelId(model: string): string {
  const n = model.trim().toLowerCase();
  if (!n || n === "all") return n;

  // DeepSeek
  if (n.includes("v4-pro") || n.includes("deepseek-reasoner")) return "deepseek-v4-pro";
  if (n.includes("v4-flash") || n.includes("deepseek-chat")) return "deepseek-v4-flash";

  // MiMo
  if (n.includes("mimo-v2.5-pro") || n.includes("mimo_v2.5_pro")) return "mimo-v2.5-pro";
  if (n.includes("mimo-v2.5") || n.includes("mimo_v2.5")) return "mimo-v2.5";
  if (n.includes("mimo-v2") || n.includes("mimo_v2")) return "mimo-v2";

  // GLM
  if (n.includes("glm-5.2") || n.includes("glm_5.2")) return "glm-5.2";
  if (n.includes("glm-5.1") || n.includes("glm_5.1")) return "glm-5.1";
  if (n.includes("glm-5") || n.includes("glm_5")) return "glm-5";

  // Kimi
  if (n.includes("kimi-k2.7-code") || n.includes("kimi_k2.7_code")) return "kimi-k2.7-code";
  if (n.includes("kimi-k2.6") || n.includes("kimi_k2.6")) return "kimi-k2.6";
  if (n.includes("kimi-k2") || n.includes("kimi_k2")) return "kimi-k2";

  // MiniMax
  if (n.includes("minimax-m3") || n.includes("MiniMax_m3") || n.includes("MiniMax-m3")) return "minimax-m3";
  if (n.includes("minimax-m2.7") || n.includes("MiniMax_m2.7") || n.includes("MiniMax-m2.7")) return "minimax-m2.7";
  if (n.includes("minimax-m2.5") || n.includes("MiniMax_m2.5") || n.includes("MiniMax-m2.5")) return "minimax-m2.5";

  // Qwen
  if (n.includes("qwen3.7-max")) return "qwen3.7-max";
  if (n.includes("qwen3.7-plus") && (n.includes(">256k") || n.includes("gt256k"))) return "qwen3.7-plus-gt256k";
  if (n.includes("qwen3.7-plus")) return "qwen3.7-plus-le256k";
  if (n.includes("qwen3.6-plus") && (n.includes(">256k") || n.includes("gt256k"))) return "qwen3.6-plus-gt256k";
  if (n.includes("qwen3.6-plus")) return "qwen3.6-plus-le256k";

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

  // Qwen
  if (n.includes("qwen3.7-max")) return "qwen3.7-max";
  if (n.includes("qwen3.7-plus")) return "qwen3.7-plus";
  if (n.includes("qwen3.6-plus")) return "qwen3.6-plus";
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
    "deepseek-v4-flash": "DeepSeek Flash",
    "deepseek-v4-pro": "DeepSeek V4 Pro",
    "mimo-v2.5-pro": "Mimo V2.5 Pro",
    "mimo-v2.5": "Mimo V2.5",
    "mimo-v2": "Mimo V2",
    "glm-5.2": "GLM-5.2",
    "glm-5.1": "GLM-5.1",
    "glm-5": "GLM-5",
    "kimi-k2.7-code": "Kimi K2.7 Code",
    "kimi-k2.6": "Kimi K2.6",
    "kimi-k2": "Kimi K2",
    "minimax-m3": "MiniMax M3",
    "minimax-m2.7": "MiniMax M2.7",
    "minimax-m2.5": "MiniMax M2.5",
    "qwen3.7-max": "Qwen3.7 Max",
    "qwen3.7-plus-le256k": "Qwen3.7 Plus (≤256K)",
    "qwen3.7-plus-gt256k": "Qwen3.7 Plus (>256K)",
    "qwen3.6-plus-le256k": "Qwen3.6 Plus (≤256K)",
    "qwen3.6-plus-gt256k": "Qwen3.6 Plus (>256K)",
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
  return [
    "deepseek-v4-pro",
    "mimo-v2.5-pro",
    "glm-5.1",
    "glm-5.2",
    "minimax-m3",
    "qwen3.7-max",
    "qwen3.7-plus-gt256k",
    "qwen3.6-plus-gt256k",
    "claude-opus-4",
    "o1",
    "o3-mini",
  ].includes(model);
}
