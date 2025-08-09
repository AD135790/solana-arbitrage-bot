use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::Deserialize;

use crate::TokenInfo;

// 一个简单的 /search 包装：按关键字搜（symbol/name/mint）
const URL_SEARCH: &str = "https://tokens.jup.ag/token/search?q=";

#[derive(Deserialize)]
struct V2SearchResp {
    tokens: Vec<V2Token>,
}
#[derive(Deserialize)]
struct V2Token {
    symbol: String,
    address: String,
    decimals: u8,
    #[serde(default)]
    name: Option<String>,
}

pub async fn search(query: &str) -> Result<Vec<TokenInfo>> {
    if query.trim().is_empty() {
        return Err(anyhow!("query 不能为空"));
    }
    let url = format!("{}{}", URL_SEARCH, urlencoding::encode(query));
    let client = Client::builder().build()?;
    let resp = client.get(&url).send().await?;

    if !resp.status().is_success() {
        return Err(anyhow!("HTTP {}: {}", resp.status(), resp.text().await.unwrap_or_default()));
    }
    let data: V2SearchResp = resp.json().await?;
    Ok(data.tokens.into_iter().map(|t| TokenInfo {
        symbol: t.symbol,
        mint: t.address,
        decimals: t.decimals,
        aliases: vec![],
    }).collect())
}
