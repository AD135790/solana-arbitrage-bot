use reqwest::Client;
use serde_json::Value;
use tracing::info;
use utils::{AppError, AppResult};
use crate::types::QuoteInfo;

pub async fn fetch_jupiter_quote(
    input_mint: &str,
    output_mint: &str,
    amount: u64,
    slippage_bps: u16, // 50 = 0.5%
) -> AppResult<QuoteInfo> {
    let client = Client::new();

    let res = client
        .get("https://quote-api.jup.ag/v6/quote")
        .query(&[
            ("inputMint", input_mint),
            ("outputMint", output_mint),
            ("amount", &amount.to_string()),
            ("slippageBps", &slippage_bps.to_string()),
        ])
        .send()
        .await?
        .error_for_status()?;

    let text = res.text().await?;
    info!("📉 Jupiter quote 响应原文: {}", text);

    let json: Value = serde_json::from_str(&text)
        .map_err(|e| AppError::ParseError(format!("❌ quote JSON 解析失败: {}", e)))?;

    let first = json.get("data")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .ok_or_else(|| AppError::Custom("❌ quote 响应缺少 data[0]".into()))?;

    let out_amount = first.get("outAmount")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Custom("❌ quote 缺少 outAmount 字段".into()))?;

    let label = first.get("routePlan")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(|v| v.get("swapInfo"))
        .and_then(|v| v.get("label"))
        .and_then(|v| v.as_str())
        .unwrap_or("未知路由");

    Ok(QuoteInfo { out_amount: out_amount.to_string(), label: label.to_string() })
}
