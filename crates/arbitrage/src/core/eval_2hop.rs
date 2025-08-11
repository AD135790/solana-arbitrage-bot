use futures::{stream, StreamExt};
use crate::ports::quote::QuoteProvider;
use crate::ports::resolver::MintResolver;
use super::types::{ArbRow, amount_from_ui};

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

                if delta_bps < min_profit_bps as f64 { return None; }

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
