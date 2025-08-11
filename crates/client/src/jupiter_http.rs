use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct QuoteReq {
    pub input_mint: String,   // 注意：用 String，调用处用 .to_string() 或 .clone()
    pub output_mint: String,
    pub amount: u64,
}

#[derive(Debug, Clone)]
pub struct QuoteResp {
    pub out_amount: u64,
}

#[async_trait::async_trait]
pub trait QuoteProvider: Send + Sync {
    async fn quote(&self, req: QuoteReq) -> Result<QuoteResp>;
}

/// 具体的 HTTP 实现（Jupiter v6）
#[derive(Clone)]
pub struct JupiterHttp {
    client: Client,
    /// 例如 50 = 0.5%
    pub slippage_bps: u16,
}

impl JupiterHttp {
    pub fn new(slippage_bps: u16) -> Self {
        Self {
            client: Client::new(),
            slippage_bps,
        }
    }
}

#[derive(Debug, Deserialize)]
struct ApiQuote {
    #[serde(rename = "outAmount")]
    out_amount: String,
}

#[async_trait::async_trait]
impl QuoteProvider for JupiterHttp {
    async fn quote(&self, req: QuoteReq) -> Result<QuoteResp> {
        let url = format!(
            "https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}",
            req.input_mint, req.output_mint, req.amount, self.slippage_bps
        );

        let resp = self.client.get(&url).send().await?.error_for_status()?;
        let payload: ApiQuote = resp.json().await?;
        let out_amount = payload
            .out_amount
            .parse::<u64>()
            .map_err(|_| anyhow!("❌ quote 缺少或无法解析 outAmount"))?;

        Ok(QuoteResp { out_amount })
    }
}

/// 保留你使用的路径：crate::jupiter_client::api::...
pub mod api {
    pub use super::{JupiterHttp, QuoteProvider, QuoteReq};
}
