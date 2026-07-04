import type { DataSource, DailyUsage, NormalizedUsage, RawUsageRecord } from "../types";

/** 规整模型名（与 Rust `canonical_model` 对齐） */
function canonicalModel(name: string): string {
  const n = name.trim().toLowerCase();
  if (!n) return "unknown";

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
  if (n.includes("minimax-m3") || n.includes("minimax-m3") || n.includes("minimax_m3"))
    return "minimax-m3";
  if (n.includes("minimax-m2.7") || n.includes("minimax-m2.7") || n.includes("minimax_m2.7"))
    return "minimax-m2.7";
  if (n.includes("minimax-m2.5") || n.includes("minimax-m2.5") || n.includes("minimax_m2.5"))
    return "minimax-m2.5";
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
  // Llama
  if (n.includes("llama-4")) return "llama-4";
  if (n.includes("llama")) return "llama-3";
  // Qwen
  if (n.includes("qwen3.7-max")) return "qwen3.7-max";
  if (n.includes("qwen3.7-plus") && (n.includes(">256k") || n.includes("gt256k")))
    return "qwen3.7-plus-gt256k";
  if (n.includes("qwen3.7-plus")) return "qwen3.7-plus-le256k";
  if (n.includes("qwen3.6-plus") && (n.includes(">256k") || n.includes("gt256k")))
    return "qwen3.6-plus-gt256k";
  if (n.includes("qwen3.6-plus")) return "qwen3.6-plus-le256k";
  if (n.includes("qwen")) return "qwen";
  // Mistral
  if (n.includes("mistral") || n.includes("mixtral")) return "mistral";

  return name.trim();
}

function ymd(ts: string): string {
  // 接受 RFC3339 / ISO 时间戳，返回 YYYY-MM-DD
  if (!ts) return "";
  const t = ts.includes("T") ? ts : ts.replace(" ", "T");
  const d = new Date(t);
  if (isNaN(d.getTime())) return "";
  return d.toISOString().slice(0, 10);
}

function ym(ts: string): string {
  return ymd(ts).slice(0, 7);
}

/** 按 dataSource 过滤原始记录 */
export function filterByDataSource(
  records: RawUsageRecord[],
  dataSource: DataSource,
): RawUsageRecord[] {
  switch (dataSource) {
    case "opencode":
      // OpenCode 经 CCSwitch 调用的所有模型用量
      return records.filter((r) => r.provider_id === "_opencode_session");
    case "claude":
      // Claude Code CLI 直接调任意 endpoint（DeepSeek / OpenCode Go / Anthropic）的用量
      // 这些记录只写 ~/.claude/projects/**/*.jsonl，provider_id 由后端标记为 "_claude_log"
      return records.filter((r) => r.provider_id === "_claude_log");
  }
}

/** 把原始记录聚合为面板展示结构 */
export function aggregateUsage(records: RawUsageRecord[]): NormalizedUsage {
  const todayStr = new Date().toISOString().slice(0, 10);
  const monthStr = todayStr.slice(0, 7);

  const dailyMap = new Map<string, DailyUsage>();
  const modelMap = new Map<string, DailyUsage>();
  let monthCost = 0;
  let monthTokens = 0;
  let monthRequests = 0;

  for (const r of records) {
    const date = ymd(r.timestamp);
    if (!date) continue;
    const model = canonicalModel(r.model);

    const inputCacheMiss = Math.max(
      0,
      r.input_tokens - r.cache_read_tokens - r.cache_creation_tokens,
    );

    // 每日
    let daily = dailyMap.get(date);
    if (!daily) {
      daily = {
        date,
        total_tokens: 0,
        input_tokens: 0,
        input_cache_hit_tokens: 0,
        input_cache_miss_tokens: 0,
        output_tokens: 0,
        request_count: 0,
        cost: 0,
        model: "all",
      };
      dailyMap.set(date, daily);
    }
    daily.total_tokens += r.total_tokens;
    daily.input_tokens += r.input_tokens;
    daily.input_cache_hit_tokens = (daily.input_cache_hit_tokens ?? 0) + r.cache_read_tokens;
    daily.input_cache_miss_tokens = (daily.input_cache_miss_tokens ?? 0) + inputCacheMiss;
    daily.output_tokens += r.output_tokens;
    daily.request_count += r.request_count;
    daily.cost += r.cost;

    // 模型
    let modelItem = modelMap.get(model);
    if (!modelItem) {
      modelItem = {
        date: todayStr,
        total_tokens: 0,
        input_tokens: 0,
        input_cache_hit_tokens: 0,
        input_cache_miss_tokens: 0,
        output_tokens: 0,
        request_count: 0,
        cost: 0,
        model,
      };
      modelMap.set(model, modelItem);
    }
    modelItem.total_tokens += r.total_tokens;
    modelItem.input_tokens += r.input_tokens;
    modelItem.input_cache_hit_tokens = (modelItem.input_cache_hit_tokens ?? 0) + r.cache_read_tokens;
    modelItem.input_cache_miss_tokens = (modelItem.input_cache_miss_tokens ?? 0) + inputCacheMiss;
    modelItem.output_tokens += r.output_tokens;
    modelItem.request_count += r.request_count;
    modelItem.cost += r.cost;

    // 月度
    if (ym(r.timestamp) === monthStr) {
      monthCost += r.cost;
      monthTokens += r.total_tokens;
      monthRequests += r.request_count;
    }
  }

  const daily = Array.from(dailyMap.values()).sort((a, b) =>
    a.date.localeCompare(b.date),
  );
  const models = Array.from(modelMap.values()).filter(
    (m) => m.total_tokens > 0 || m.cost > 0,
  );

  return {
    daily,
    models,
    monthly: {
      total_cost: monthCost,
      currency: "USD",
      month: monthStr,
      total_tokens: monthTokens,
      request_count: monthRequests,
    },
    has_daily_granularity: daily.length > 0,
  };
}
