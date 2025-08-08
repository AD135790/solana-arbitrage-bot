use reqwest::Client;
use serde_json::Value;
use tracing::info;
use utils::{AppError, AppResult};
use crate::types::QuoteInfo;

/// Jupiter 报价接口：返回 QuoteInfo（用于 /quote）
pub async fn fetch_jupiter_quote(
    input_mint: &str,
    output_mint: &str,
    amount: u64,
) -> AppResult<QuoteInfo> {
    let url = format!(
        "https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}",
        input_mint, output_mint, amount
    );

    let client = Client::new();
    let res = client.get(&url).send().await?;
    let text = res.text().await?;

    info!("📉 Jupiter quote 响应原文: {}", text);

    let json: Value = serde_json::from_str(&text)
        .map_err(|e| AppError::ParseError(format!("❌ quote JSON 解析失败: {}", e)))?;

    let out_amount = json
        .get("outAmount")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Custom("❌ quote 缺少 outAmount 字段".into()))?;

    let label = json
        .get("routePlan")
        .and_then(|v| v.get(0))
        .and_then(|v| v.get("swapInfo"))
        .and_then(|v| v.get("label"))
        .and_then(|v| v.as_str())
        .unwrap_or("未知路由");

    Ok(QuoteInfo {
        out_amount: out_amount.to_string(),
        label: label.to_string(),
    })
}
