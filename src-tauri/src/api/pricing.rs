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
    ModelPricing { name: "deepseek-v4-flash", input: 0.14, cache_write: 0.175, cache_read: 0.07, output: 0.28 },
    ModelPricing { name: "deepseek-v4-pro", input: 0.42, cache_write: 0.525, cache_read: 0.21, output: 0.84 },
    ModelPricing { name: "deepseek-chat", input: 0.14, cache_write: 0.175, cache_read: 0.07, output: 0.28 },
    ModelPricing { name: "deepseek-reasoner", input: 0.42, cache_write: 0.525, cache_read: 0.21, output: 0.84 },
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
/// 费用 = 普通输入 * input_price + 缓存创建 * cache_write_price
///        + 缓存命中 * cache_read_price + 输出 * output_price
pub fn calculate_cost(
    model: &str,
    input_tokens: u64,
    cache_read_tokens: u64,
    cache_creation_tokens: u64,
    output_tokens: u64,
) -> f64 {
    let p = lookup(model);

    let standard_input = (input_tokens as i64)
        .saturating_sub(cache_read_tokens as i64)
        .saturating_sub(cache_creation_tokens as i64)
        .max(0) as u64;

    let cost = (standard_input as f64 * p.input
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
        let p = lookup("claude-sonnet-4-6");
        assert!((p.input - 3.0).abs() < 0.001);
        let p = lookup("unknown-model");
        assert!((p.input - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_cost() {
        let cost = calculate_cost("deepseek-v4-flash", 1000, 0, 0, 500);
        assert!(cost > 0.0);
    }
}
