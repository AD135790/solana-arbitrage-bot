// crates/arbitrage/src/test/eval_smoke.rs
use std::collections::HashMap;
use anyhow::Result;
use arbitrage::prelude::*;
use tokio::runtime::Runtime;

// ---- Mock：报价器（按倍数返回）----
struct MapQuoter {
    m: HashMap<(String, String), f64>,
}
impl MapQuoter {
    fn new() -> Self { Self { m: HashMap::new() } }
    fn set(mut self, a: &str, b: &str, k: f64) -> Self {
        self.m.insert((a.to_string(), b.to_string()), k); self
    }
}
#[allow(async_fn_in_trait)]
impl QuoteProvider for MapQuoter {
    async fn quote(&self, im: String, om: String, amount: u64) -> Result<u64> {
        let k = *self.m.get(&(im.clone(), om.clone())).unwrap_or(&1.0);
        Ok(((amount as f64) * k).round() as u64)
    }
}

// ---- Mock：Resolver（符号 → mint + decimals）----
struct DummyResolver;
impl MintResolver for DummyResolver {
    fn get_mint(&self, sym: &str) -> anyhow::Result<String> {
        // 简单映射：大写即 mint
        Ok(sym.to_uppercase())
    }
    fn get_decimals(&self, _sym: &str) -> Option<u8> { Some(9) } // lamports 风格
    // 如果 trait 还有别的方法，按需要补一个“保守返回”
}

// ---- 2-hop 冒烟测试：SOL → USDC → SOL，期望约 200 bps ----
#[test]
fn test_2hop_positive_bps() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let resolver = DummyResolver;

        // 在 test 里补一段
let raw = MapQuoter::new().set("SOL","USDC", 1.0).set("USDC","SOL",1.0);
let mw  = ThrottleRetry { qps: 10, retries: 2 };
let wrapped = mw.wrap(&raw);
let _rows = evaluate_2hop(&DummyResolver, &wrapped, "SOL", &vec!["USDC".into()], 1.0, i32::MIN, 4).await;

        // 1 SOL = 1e9；设：SOL→USDC *2.0，USDC→SOL *0.51  => 约 1.02 SOL
        let quoter = MapQuoter::new()
            .set("SOL", "USDC", 2.0)
            .set("USDC", "SOL", 0.51);

        let mids = vec!["USDC".to_string()];
        let rows = evaluate_2hop(&resolver, &quoter, "SOL", &mids, 1.0, i32::MIN, 4).await;

        assert_eq!(rows.len(), 1);
        let r = &rows[0];
        assert!(r.end > 1.0);
        assert!(r.delta_bps > 190.0, "delta_bps={}", r.delta_bps);
        assert_eq!(r.path, "SOL → USDC → SOL");
    });
}

// ---- 3-hop 冒烟测试：SOL → A → B → SOL 正常返回 ----
#[test]
fn test_3hop_smoke() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let resolver = DummyResolver;

        let quoter = MapQuoter::new()
            .set("SOL","A", 1.2)
            .set("A","B",  1.1)
            .set("B","SOL",0.8);

        let mids = vec!["A".to_string(), "B".to_string()];
        let rows = evaluate_3hop(&resolver, &quoter, "SOL", &mids, 1.0, 4, false).await;

        assert!(!rows.is_empty());
        assert!(rows.iter().any(|r| r.path == "SOL → A → B → SOL"));
    });
}
