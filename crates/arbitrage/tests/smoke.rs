use arbitrage::prelude::*;
use anyhow::Result;
use std::collections::HashMap;

/* ---- Mock 报价器 ---- */
struct MapQuoter { m: HashMap<(String,String), f64> }
impl MapQuoter {
    fn new() -> Self { Self { m: HashMap::new() } }
    fn set(mut self, a:&str,b:&str,k:f64)->Self{ self.m.insert((a.into(),b.into()),k); self }
}
#[allow(async_fn_in_trait)]
impl QuoteProvider for MapQuoter {
    async fn quote(&self, im:String, om:String, amount:u64) -> Result<u64> {
        let k = *self.m.get(&(im,om)).unwrap_or(&1.0);
        Ok(((amount as f64)*k).round() as u64)
    }
}

/* ---- Mock Resolver ---- */
struct DummyResolver;
impl MintResolver for DummyResolver {
    fn get_mint(&self, sym: &str) -> anyhow::Result<&str> {
        match sym.to_ascii_uppercase().as_str() {
            "SOL"  => Ok("So11111111111111111111111111111111111111112"),
            "USDC" => Ok("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
            "A"    => Ok("A111111111111111111111111111111111111111111"),
            "B"    => Ok("B111111111111111111111111111111111111111111"),
            _ => anyhow::bail!("unknown"),
        }
    }
    fn get_decimals(&self, sym: &str) -> Option<u8> {
        match sym.to_ascii_uppercase().as_str() { "USDC" => Some(6), _ => Some(9) }
    }
    fn is_tradable(&self, _mint: &str) -> Option<bool> { Some(true) }
}

/* ---- 2-hop ---- */
#[tokio::test]
async fn two_hop_smoke() {
    let resolver = DummyResolver;
    let sol = "So11111111111111111111111111111111111111112";
    let usdc = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

    let quoter = MapQuoter::new()
        .set(sol,  usdc, 2.0)
        .set(usdc, sol,  0.51);

    let mids = vec!["USDC".to_string()];
    let rows = evaluate_2hop(&resolver, &quoter, "SOL", &mids, 1.0, i32::MIN, 4).await;

    assert_eq!(rows.len(), 1);
    assert!(rows[0].end > 1.0);
    assert!(rows[0].delta_bps > 190.0);
    assert_eq!(rows[0].path, "SOL → USDC → SOL");
}

/* ---- 3-hop ---- */
#[tokio::test]
async fn three_hop_smoke() {
    let resolver = DummyResolver;
    let sol = "So11111111111111111111111111111111111111112";
    let a   = "A111111111111111111111111111111111111111111";
    let b   = "B111111111111111111111111111111111111111111";

    let quoter = MapQuoter::new()
        .set(sol, a, 1.2)
        .set(a,   b, 1.1)
        .set(b, sol, 0.8);

    let mids = vec!["A".to_string(), "B".to_string()];
    let rows = evaluate_3hop(&resolver, &quoter, "SOL", &mids, 1.0, 4, false).await;

    assert!(!rows.is_empty());
    assert!(rows.iter().any(|r| r.path == "SOL → A → B → SOL"));
}
