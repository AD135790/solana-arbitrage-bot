use anyhow::{anyhow, Result};
use reqwest::Client;
use std::collections::HashSet;

/// 官方 v1（deprecated，但仍能用）：返回 Vec<String> mint
const URL: &str = "https://lite-api.jup.ag/tokens/v1/mints/tradable";

pub async fn fetch_supported_mints() -> Result<HashSet<String>> {
    let client = Client::builder().build()?;
    let resp = client.get(URL).send().await?;

    if !resp.status().is_success() {
        return Err(anyhow!("HTTP {}: {}", resp.status(), resp.text().await.unwrap_or_default()));
    }
    let mints: Vec<String> = resp.json().await?;
    Ok(mints.into_iter().collect())
}
