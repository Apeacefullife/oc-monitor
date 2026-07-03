/// OpenCode Go 订阅定价表（每百万 tokens 的价格，单位 USD）
///
/// 数据来源：https://opencode.ai/pricing
/// 模型名匹配：不区分大小写，使用子串匹配

/// 模型定价
pub struct ModelPricing {
    /// 模型名（显示用）
    pub name: &'static str,
    /// 输入价格（$ / 1M tokens）
    pub input: f64,
    /// 缓存写入/创建价格（$ / 1M tokens）
    pub cache_write: f64,
    /// 缓存命中价格（$ / 1M tokens）
    pub cache_read: f64,
    /// 输出价格（$ / 1M tokens）
    pub output: f64,
}

const PRICING: &[ModelPricing] = &[
    // Anthropic Claude
    ModelPricing { name: "claude-sonnet-4-7", input: 3.00, cache_write: 3.75, cache_read: 0.30, output: 15.00 },
    ModelPricing { name: "claude-sonnet-4-6", input: 3.00, cache_write: 3.75, cache_read: 0.30, output: 15.00 },
    ModelPricing { name: "claude-sonnet-4", input: 3.00, cache_write: 3.75, cache_read: 0.30, output: 15.00 },
    ModelPricing { name: "claude-opus-4-7", input: 15.00, cache_write: 18.75, cache_read: 1.50, output: 75.00 },
    ModelPricing { name: "claude-opus-4", input: 15.00, cache_write: 18.75, cache_read: 1.50, output: 75.00 },
    ModelPricing { name: "claude-haiku-4-5", input: 0.80, cache_write: 1.00, cache_read: 0.08, output: 4.00 },
    ModelPricing { name: "claude-haiku-3-5", input: 0.80, cache_write: 1.00, cache_read: 0.08, output: 4.00 },
    // DeepSeek
    ModelPricing { name: "deepseek-v4-flash", input: 0.14, cache_write: 0.0, cache_read: 0.0028, output: 0.28 },
    ModelPricing { name: "deepseek-v4-pro", input: 1.74, cache_write: 0.0, cache_read: 0.0145, output: 3.48 },
    ModelPricing { name: "deepseek-chat", input: 0.14, cache_write: 0.0, cache_read: 0.0028, output: 0.28 },
    ModelPricing { name: "deepseek-reasoner", input: 1.74, cache_write: 0.0, cache_read: 0.0145, output: 3.48 },
    // MiMo
    ModelPricing { name: "mimo-v2.5-pro", input: 1.74, cache_write: 0.0, cache_read: 0.0145, output: 3.48 },
    ModelPricing { name: "mimo-v2.5", input: 0.14, cache_write: 0.0, cache_read: 0.0028, output: 0.28 },
    ModelPricing { name: "mimo-v2", input: 0.14, cache_write: 0.0, cache_read: 0.0028, output: 0.28 },
    // GLM
    ModelPricing { name: "glm-5.2", input: 1.40, cache_write: 0.0, cache_read: 0.26, output: 4.40 },
    ModelPricing { name: "glm-5.1", input: 1.40, cache_write: 0.0, cache_read: 0.26, output: 4.40 },
    ModelPricing { name: "glm-5", input: 1.40, cache_write: 0.0, cache_read: 0.26, output: 4.40 },
    // Kimi
    ModelPricing { name: "kimi-k2.7-code", input: 0.95, cache_write: 0.0, cache_read: 0.19, output: 4.00 },
    ModelPricing { name: "kimi-k2.6", input: 0.95, cache_write: 0.0, cache_read: 0.16, output: 4.00 },
    ModelPricing { name: "kimi-k2", input: 0.95, cache_write: 0.0, cache_read: 0.16, output: 4.00 },
    // Qwen
    ModelPricing { name: "qwen3.7-max", input: 2.50, cache_write: 3.125, cache_read: 0.50, output: 7.50 },
    ModelPricing { name: "qwen3.7-plus-le256k", input: 0.40, cache_write: 0.50, cache_read: 0.04, output: 1.60 },
    ModelPricing { name: "qwen3.7-plus-gt256k", input: 1.20, cache_write: 1.50, cache_read: 0.12, output: 4.80 },
    ModelPricing { name: "qwen3.6-plus-le256k", input: 0.50, cache_write: 0.625, cache_read: 0.05, output: 3.00 },
    ModelPricing { name: "qwen3.6-plus-gt256k", input: 2.00, cache_write: 2.50, cache_read: 0.20, output: 6.00 },
    // MiniMax
    ModelPricing { name: "minimax-m3", input: 0.30, cache_write: 0.0, cache_read: 0.06, output: 1.20 },
    ModelPricing { name: "minimax-m2.7", input: 0.30, cache_write: 0.375, cache_read: 0.06, output: 1.20 },
    ModelPricing { name: "minimax-m2.5", input: 0.30, cache_write: 0.375, cache_read: 0.06, output: 1.20 },
    // OpenAI
    ModelPricing { name: "gpt-4o", input: 2.50, cache_write: 3.125, cache_read: 0.625, output: 10.00 },
    ModelPricing { name: "gpt-4o-mini", input: 0.15, cache_write: 0.1875, cache_read: 0.075, output: 0.60 },
    ModelPricing { name: "o1", input: 15.00, cache_write: 15.00, cache_read: 7.50, output: 60.00 },
    ModelPricing { name: "o3-mini", input: 1.10, cache_write: 1.375, cache_read: 0.55, output: 4.40 },
    // Google Gemini
    ModelPricing { name: "gemini-2.5-flash", input: 0.15, cache_write: 0.1875, cache_read: 0.0375, output: 0.60 },
    ModelPricing { name: "gemini-2.5-pro", input: 1.25, cache_write: 1.5625, cache_read: 0.3125, output: 5.00 },
    ModelPricing { name: "gemini-2.0-flash", input: 0.10, cache_write: 0.125, cache_read: 0.025, output: 0.40 },
    // Meta Llama
    ModelPricing { name: "llama-4", input: 0.25, cache_write: 0.25, cache_read: 0.0625, output: 0.75 },
    ModelPricing { name: "llama-3.3", input: 0.25, cache_write: 0.25, cache_read: 0.0625, output: 0.75 },
    ModelPricing { name: "llama-3.1", input: 0.25, cache_write: 0.25, cache_read: 0.0625, output: 0.75 },
    // 通义千问
    ModelPricing { name: "qwen", input: 0.50, cache_write: 0.50, cache_read: 0.125, output: 1.50 },
    // Mistral
    ModelPricing { name: "mistral", input: 0.20, cache_write: 0.20, cache_read: 0.05, output: 0.60 },
    // 默认兜底
    ModelPricing { name: "__default__", input: 1.00, cache_write: 1.00, cache_read: 0.50, output: 3.00 },
];

const PRICING_FALLBACK: &ModelPricing = &PRICING[PRICING.len() - 1];

/// 根据模型名查找定价
pub fn lookup(model: &str) -> &'static ModelPricing {
    let lower = model.trim().to_lowercase();
    for p in PRICING.iter().take(PRICING.len() - 1) {
        if lower.contains(p.name) {
            return p;
        }
    }
    PRICING_FALLBACK
}

/// 计算单次请求的费用（美元）
///
/// 计费方式（OpenCode 模型）：
/// - input_tokens 按 input_price 计费
/// - cache_read_tokens 按 cache_read_price 计费（命中缓存的折扣价）
/// - cache_creation_tokens 按 cache_write_price 计费
/// - output_tokens 按 output_price 计费
///
/// 注意：input_tokens 与 cache_read_tokens 互相独立计费（不互相抵消），
/// 这与 CCSwitch 数据库中 total_cost_usd 的计算方式一致。
pub fn calculate_cost(
    model: &str,
    input_tokens: u64,
    cache_read_tokens: u64,
    cache_creation_tokens: u64,
    output_tokens: u64,
) -> f64 {
    let p = lookup(model);

    let cost = (input_tokens as f64 * p.input
        + cache_creation_tokens as f64 * p.cache_write
        + cache_read_tokens as f64 * p.cache_read
        + output_tokens as f64 * p.output)
        / 1_000_000.0;

    cost
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup() {
        let p = lookup("deepseek-v4-flash");
        assert!((p.input - 0.14).abs() < 0.001);
        let p = lookup("mimo-v2.5");
        assert!((p.input - 0.14).abs() < 0.001);
        let p = lookup("deepseek-v4-pro");
        assert!((p.input - 1.74).abs() < 0.001);
        let p = lookup("unknown-model");
        assert!((p.input - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_cost() {
        let cost = calculate_cost("deepseek-v4-flash", 1000, 0, 0, 500);
        assert!(cost > 0.0);
    }
}
