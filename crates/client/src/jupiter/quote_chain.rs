use crate::jupiter::quote::fetch_jupiter_quote;
use utils::resolve_mint_address;
use utils::{AppError, AppResult};

/// 链式报价中每一步的执行信息
#[derive(Debug)]
pub struct ChainQuoteStep {
    pub from: String,
    pub to: String,
    pub input_amount: u64,
    pub output_amount: u64,
    pub label: String,
}

/// 链式报价的结果汇总
#[derive(Debug)]
pub struct ChainQuoteResult {
    pub steps: Vec<ChainQuoteStep>,
    pub final_amount: u64,
}

/// 🚀 模拟链式报价路径，例如 ["SOL", "USDC", "MSOL", "SOL"]
pub async fn fetch_chain_quotes(
    path: Vec<&str>,
    start_amount: u64,
) -> AppResult<ChainQuoteResult> {
    if path.len() < 2 {
        return Err(AppError::Custom("路径至少包含两个币种".into()));
    }

    let mut steps = Vec::new();
    let mut current_amount = start_amount;

    for i in 0..(path.len() - 1) {
        let from = path[i].to_uppercase();
        let to = path[i + 1].to_uppercase();

        let input_mint = resolve_mint_address(&from)
            .ok_or_else(|| AppError::Custom(format!("❌ 无法识别币种: {}", from)))?;
        let output_mint = resolve_mint_address(&to)
            .ok_or_else(|| AppError::Custom(format!("❌ 无法识别币种: {}", to)))?;

        let quote = fetch_jupiter_quote(input_mint, output_mint, current_amount).await?;

        let out_amount: u64 = quote.out_amount.parse().map_err(|_| {
            AppError::Custom(format!("❌ 无法解析 out_amount: {}", quote.out_amount))
        })?;

        steps.push(ChainQuoteStep {
            from,
            to,
            input_amount: current_amount,
            output_amount: out_amount,
            label: quote.label,
        });

        current_amount = out_amount;
    }

    Ok(ChainQuoteResult {
        steps,
        final_amount: current_amount,
    })
}
