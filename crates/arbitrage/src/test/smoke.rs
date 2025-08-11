use arbitrage::prelude::*;
use anyhow::Result;
use std::collections::HashMap;

/* ---------------- Mock 报价器 ---------------- */

struct MapQuoter {
    m: HashMap<(String, String), f64>,
}

impl MapQuoter {
    fn new() -> Self { Self { m: HashMap::new() } }
    fn set(mut self, a: &str, b: &str, k: f64) -> Self {
        self.m.insert((a.to_string(), b.to_string()), k);
        self
    }
}

#[allow(async_fn_in_trait)]
impl QuoteProvider for MapQuoter {
    async fn quote(&self, im: String, om: String, amount: u64) -> Result<u64> {
        let k = *self.m.get(&(im, om)).unwrap_or(&1.0);
        Ok(((amount as f64) * k).round() as u64)
    }
}

/* ---------------- Mock 解析器 ---------------- */

struct DummyResolver;

impl MintResolver for DummyResolver {
    // 注意：这里返回 &str（静态），以匹配 trait 要求
    fn get_mint(&self, sym: &str) -> anyhow::Result<&str> {
        match sym.to_ascii_uppercase().as_str() {
            "SOL"  => Ok("So11111111111111111111111111111111111111112"),
            "USDC" => Ok("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
            "A"    => Ok("A111111111111111111111111111111111111111111"),
            "B"    => Ok("B111111111111111111111111111111111111111111"),
            other  => anyhow::bail!("unknown symbol: {}", other),
        }
    }

    fn get_decimals(&self, sym: &str) -> Option<u8> {
        match sym.to_ascii_uppercase().as_str() {
            "USDC" => Some(6),
            _      => Some(9), // 其余按 9
        }
    }

    fn is_tradable(&self, _mint: &str) -> Option<bool> {
        Some(true)
    }
}

/* ---------------- 冒烟用例 ---------------- */

#[tokio::test]
async fn two_hop_smoke() {
    let resolver = DummyResolver;

    // 1 SOL -> 2 USDC -> 1.02 SOL （约 200 bps）
    let quoter = MapQuoter::new()
        .set("So11111111111111111111111111111111111111112", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", 2.0)
        .set("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "So11111111111111111111111111111111111111112", 0.51);

    let mids = vec!["USDC".to_string()];
    let rows = evaluate_2hop(&resolver, &quoter, "SOL", &mids, 1.0, i32::MIN, 4).await;

    assert_eq!(rows.len(), 1, "应该有一条路径");
    assert!(rows[0].end > 1.0, "end={}", rows[0].end);
    assert!(rows[0].delta_bps > 190.0, "bps={}", rows[0].delta_bps);
    assert_eq!(rows[0].path, "SOL → USDC → SOL");
}

#[tokio::test]
async fn three_hop_smoke() {
    let resolver = DummyResolver;

    let quoter = MapQuoter::new()
        .set("So11111111111111111111111111111111111111112", "A111111111111111111111111111111111111111111", 1.2)
        .set("A111111111111111111111111111111111111111111", "B111111111111111111111111111111111111111111", 1.1)
        .set("B111111111111111111111111111111111111111111", "So11111111111111111111111111111111111111112", 0.8);

    let mids = vec!["A".to_string(), "B".to_string()];
    let rows = evaluate_3hop(&resolver, &quoter, "SOL", &mids, 1.0, 4, false).await;

    assert!(!rows.is_empty(), "3-hop 应该返回至少一条");
    assert!(rows.iter().any(|r| r.path == "SOL → A → B → SOL"));
}

/* ---------------- 中间件连通性（可选） ---------------- */

#[tokio::test]
async fn throttle_retry_compiles_and_runs() {
    let resolver = DummyResolver;
    let raw = MapQuoter::new()
        .set("So11111111111111111111111111111111111111112", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", 1.0)
        .set("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "So11111111111111111111111111111111111111112", 1.0);

    let mw = ThrottleRetry { qps: 50, retries: 1 };
    let wrapped = mw.wrap(&raw);

    let mids = vec!["USDC".to_string()];
    let rows = evaluate_2hop(&resolver, &wrapped, "SOL", &mids, 1.0, i32::MIN, 2).await;
    assert_eq!(rows.len(), 1);
}
