use std::collections::HashSet;
use utils::{AppResult, AppError};

pub async fn fetch_supported_tokens() -> AppResult<HashSet<String>> {
    let url = "https://lite-api.jup.ag/tokens/v1/mints/tradable";

    let resp = reqwest::get(url)
        .await
        .map_err(|e| AppError::Custom(format!("请求失败: {}", e)))?;

    if !resp.status().is_success() {
        return Err(AppError::Custom(format!("HTTP {}: {}", resp.status(), resp.text().await.unwrap_or_default())));
    }

    // ⚠️ 这里直接就是 `Vec<String>`，不是对象数组
    let mints: Vec<String> = resp
        .json()
        .await
        .map_err(|e| AppError::Custom(format!("解析 mint 列表失败: {}", e)))?;

    Ok(mints.into_iter().collect())
}
