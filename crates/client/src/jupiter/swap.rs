use reqwest::Client;
use serde_json::Value;
use tracing::{info, error};
use utils::{AppError, AppResult};

/// Jupiter swap 接口：返回交易体 base64（用于执行 swap）
pub async fn fetch_jupiter_swap_tx_safe(
    input_mint: &str,
    output_mint: &str,
    amount: f64,
    input_decimals: u8,
    user_pubkey: &str,
) -> AppResult<String> {
    if amount <= 0.0 {
        return Err(AppError::Custom("❌ amount 必须大于 0".into()));
    }

    let amount_u64 = (amount * 10f64.powi(input_decimals as i32)) as u64;

    info!(
        "💱 请求 Jupiter Swap: {} {} -> {} by {}",
        amount, input_mint, output_mint, user_pubkey
    );

    let url = "https://quote-api.jup.ag/v6/swap";
    let body = serde_json::json!({
        "inputMint": input_mint,
        "outputMint": output_mint,
        "amount": amount_u64.to_string(),
        "slippageBps": 50,
        "userPublicKey": user_pubkey,
        "wrapUnwrapSol": true,
        "feeBps": 0
    });

    let client = Client::new();
    let resp = client.post(url).json(&body).send().await?;
    let text = resp.text().await?;

    if text.contains("\"error\"") {
        error!("🚫 Jupiter swap 请求失败，原始返回: {}", text);
        return Err(AppError::External(format!("Jupiter 返回错误: {}", text)));
    }

    let json: Value = serde_json::from_str(&text)
        .map_err(|e| AppError::ParseError(format!("❌ swap JSON 解析失败: {}", e)))?;

    info!("🔃 Jupiter swap 响应 JSON: {}", json);

    let tx = json
        .get("swapTransaction")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Custom("❌ Jupiter 未返回 swapTransaction 字段".into()))?;

    Ok(tx.to_string())
}
