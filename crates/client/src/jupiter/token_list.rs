use serde::Deserialize;
use std::collections::HashSet;
use utils::{AppResult, AppError};

#[derive(Debug, Deserialize)]
pub struct JupiterToken {
    pub address: String,
    pub symbol: String,
}

/// ✅ 从 Jupiter 获取所有支持的 token mint 列表
pub async fn fetch_supported_tokens() -> AppResult<HashSet<String>> {
    let url = "https://quote-api.jup.ag/v4/tokens";

    let resp = reqwest::get(url)
        .await
        .map_err(|e| AppError::Custom(format!("请求失败: {}", e)))?;

    let body = resp
        .text()
        .await
        .map_err(|e| AppError::Custom(format!("读取响应失败: {}", e)))?;

    // ✅ 正确：直接解析为 Vec<JupiterToken>
    let token_list: Vec<JupiterToken> = serde_json::from_str(&body)
        .map_err(|e| AppError::Custom(format!("解析 token list 失败: {}", e)))?;

    let supported: HashSet<String> = token_list
        .into_iter()
        .map(|t| t.address)
        .collect();

    Ok(supported)
}
