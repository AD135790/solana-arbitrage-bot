use futures::{stream, StreamExt};
use crate::ports::quote::QuoteProvider;
use crate::ports::resolver::MintResolver;
use super::types::{ArbRow, amount_from_ui};

pub async fn evaluate_3hop<R, Q>(
    resolver: &R,
    quoter: &Q,
    base: &str,
    mids: &[String],
    ui_amount: f64,
    concurrency: usize,
    _verbose: bool, // 不在策略层打印
) -> Vec<ArbRow>
where
    R: MintResolver + Sync,
    Q: QuoteProvider + Sync,
{
    let base_uc   = base.to_uppercase();
    let base_mint = match resolver.get_mint(&base_uc) { Ok(m) => m, Err(_) => return vec![] };
    let base_dec  = resolver.get_decimals(&base_uc).unwrap_or(9);
    let start_amt = amount_from_ui(base_dec, ui_amount);

    let mut parsed: Vec<(String, String)> = Vec::new();
    for s in mids {
        if let Ok(m) = resolver.get_mint(s) {
            parsed.push((s.to_uppercase(), m.to_string()));
        }
    }

    let pairs: Vec<((String,String),(String,String))> = parsed.iter()
        .flat_map(|a| parsed.iter().filter(move |b| !std::ptr::eq(*b, a)).map(|b| (a.clone(), b.clone())))
        .collect();

    stream::iter(pairs.into_iter())
        .map(|((a_sym, a_mint), (b_sym, b_mint))| {
            let base_uc = base_uc.clone();
            async move {
                let out1 = quoter.quote(base_mint.to_string(), a_mint.clone(), start_amt).await.ok()?;
                let out2 = quoter.quote(a_mint.clone(), b_mint.clone(), out1).await.ok()?;
                let out3 = quoter.quote(b_mint.clone(), base_mint.to_string(), out2).await.ok()?;

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
