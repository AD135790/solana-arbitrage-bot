use reqwest::Client;
use serde_json::Value;
use utils::{AppError, AppResult};
use crate::types::QuoteRoute;

/// 获取所有路线（用于 --verbose 模式和套利分析）
pub async fn fetch_jupiter_routes(
    input_mint: &str,
    output_mint: &str,
    amount: u64,
) -> AppResult<Vec<QuoteRoute>> {
    let url = format!(
        "https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}",
        input_mint, output_mint, amount
    );

    let client = Client::new();
    let res = client.get(&url).send().await?;
    let text = res.text().await?;

    let json: Value = serde_json::from_str(&text)
        .map_err(|e| AppError::ParseError(format!("❌ JSON 解析失败: {}", e)))?;

    let out_amount = json.get("outAmount").and_then(|v| v.as_str()).unwrap_or("0").to_string();
    let label = json
        .get("routePlan")
        .and_then(|v| v.get(0))
        .and_then(|v| v.get("swapInfo"))
        .and_then(|v| v.get("label"))
        .and_then(|v| v.as_str())
        .unwrap_or("未知")
        .to_string();

    let hops = json
        .get("routePlan")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    Ok(vec![QuoteRoute {
        out_amount,
        label,
        hops,
    }])
}
