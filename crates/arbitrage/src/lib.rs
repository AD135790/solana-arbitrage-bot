#![allow(async_fn_in_trait)]

use tokio::time::{sleep, Duration};
use anyhow::Result;
use futures::{stream, StreamExt};
use serde::Serialize;
use token_registry::api::MintResolver;

/// 上层（CLI/Bot）的报价器适配到这个 trait 即可
pub trait QuoteProvider: Send + Sync {
    async fn quote(&self, input_mint: String, output_mint: String, amount: u64) -> Result<u64>;
}

#[derive(Debug, Serialize, Clone)]
pub struct ArbRow {
    pub profitable: bool,
    pub path: String,
    pub start: f64,     // UI 金额
    pub end: f64,       // UI 金额
    pub delta_bps: f64, // (end/start - 1) * 10_000
}

fn amount_from_ui(decimals: u8, ui: f64) -> u64 {
    (ui * 10f64.powi(decimals as i32)).round() as u64
}

/// 评估 base→mid→base（并发可控，带最小盈利过滤）
pub async fn evaluate_2hop<R, Q>(
    resolver: &R,
    quoter: &Q,
    base: &str,
    mids: &[String],
    ui_amount: f64,
    min_profit_bps: i32,
    concurrency: usize,
) -> Vec<ArbRow>
where
    R: MintResolver + Sync,
    Q: QuoteProvider + Sync,
{
    let base_uc   = base.to_uppercase();
    let base_mint = match resolver.get_mint(&base_uc) { Ok(m) => m, Err(_) => return vec![] };
    let base_dec  = resolver.get_decimals(&base_uc).unwrap_or(9);
    let start_amt = amount_from_ui(base_dec, ui_amount);

    stream::iter(mids.iter().cloned())
        .map(|mid| {
            let base_uc = base_uc.clone();
            async move {
                let mid_mint = resolver.get_mint(&mid).ok()?;

                // base -> mid
                let out1 = quoter.quote(
                    base_mint.to_string(),
                    mid_mint.to_string(),
                    start_amt,
                ).await.ok()?;

                // mid -> base
                let out2 = quoter.quote(
                    mid_mint.to_string(),
                    base_mint.to_string(),
                    out1,
                ).await.ok()?;

                let end_ui    = (out2 as f64) / 10f64.powi(base_dec as i32);
                let ratio     = end_ui / ui_amount;
                let delta_bps = (ratio - 1.0) * 10_000.0;

                if delta_bps < min_profit_bps as f64 {
                    return None;
                }

                Some(ArbRow {
                    profitable: delta_bps >= 0.0,
                    path: format!("{} → {} → {}", base_uc, mid.to_uppercase(), base_uc),
                    start: ui_amount,
                    end: end_ui,
                    delta_bps,
                })
            }
        })
        .buffer_unordered(concurrency.max(1))
        .filter_map(|x| async move { x })
        .collect::<Vec<_>>()
        .await
}

/// 轻量重试：100ms → 250ms → 500ms；verbose 时打印失败原因
async fn quote_retry<Q: QuoteProvider>(
    quoter: &Q,
    input_mint: String,
    output_mint: String,
    amount: u64,
    verbose: bool,
    label: &str,
) -> Option<u64> {
    let backoff = [100u64, 250, 500];
    for (i, d) in backoff.iter().enumerate() {
        match quoter.quote(input_mint.clone(), output_mint.clone(), amount).await {
            Ok(x) => return Some(x),
            Err(e) => {
                if verbose {
                    eprintln!("⚠️ {} try#{} failed: {}", label, i + 1, e);
                }
                sleep(Duration::from_millis(*d)).await;
            }
        }
    }
    None
}

pub async fn evaluate_3hop<R, Q>(
    resolver: &R,
    quoter: &Q,
    base: &str,
    mids: &[String],
    ui_amount: f64,
    concurrency: usize,
    verbose: bool, // 👈 新增
) -> Vec<ArbRow>
where
    R: MintResolver + Sync,
    Q: QuoteProvider + Sync,
{
    let base_uc   = base.to_uppercase();
    let base_mint = match resolver.get_mint(&base_uc) { Ok(m) => m, Err(_) => return vec![] };
    let base_dec  = resolver.get_decimals(&base_uc).unwrap_or(9);
    let start_amt = amount_from_ui(base_dec, ui_amount);

    // 预解析 mids → (SYMBOL, mint)
    let mut parsed: Vec<(String, String)> = Vec::new();
    for s in mids {
        if let Ok(m) = resolver.get_mint(s) {
            parsed.push((s.to_uppercase(), m.to_string()));
        }
    }

    // 生成有序 (A,B)，A≠B
    let pairs: Vec<((String,String),(String,String))> = parsed.iter()
        .flat_map(|a| parsed.iter().filter(move |b| !std::ptr::eq(*b, a)).map(|b| (a.clone(), b.clone())))
        .collect();

    stream::iter(pairs.into_iter())
        .map(|((a_sym, a_mint), (b_sym, b_mint))| {
            let base_uc = base_uc.clone();
            async move {
                // base→A
                let out1 = quote_retry(quoter, base_mint.to_string(), a_mint.clone(), start_amt, verbose, "base→A").await?;
                // A→B
                let out2 = quote_retry(quoter, a_mint.clone(), b_mint.clone(), out1,       verbose, "A→B").await?;
                // B→base
                let out3 = quote_retry(quoter, b_mint.clone(), base_mint.to_string(), out2, verbose, "B→base").await?;

                let end_ui    = (out3 as f64) / 10f64.powi(base_dec as i32);
                let ratio     = end_ui / ui_amount;
                let delta_bps = (ratio - 1.0) * 10_000.0;

                Some(ArbRow {
                    profitable: delta_bps >= 0.0,
                    path: format!("{} → {} → {} → {}", base_uc, a_sym, b_sym, base_uc),
                    start: ui_amount,
                    end: end_ui,
                    delta_bps,
                })
            }
        })
        .buffer_unordered(concurrency.max(1))
        .filter_map(|x| async move { x })
        .collect::<Vec<_>>()
        .await
}
