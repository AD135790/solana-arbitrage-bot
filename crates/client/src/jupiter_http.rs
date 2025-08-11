use anyhow::{Result, anyhow};
use async_trait::async_trait;
use arbitrage::prelude::QuoteProvider as StratQuoteProvider; // 策略层接口
use crate::jupiter::quote::fetch_jupiter_quote;              // 你刚写的HTTP函数
use crate::types::QuoteInfo;

#[derive(Clone)]
pub struct JupiterHttp {
    pub client: reqwest::Client,
    pub base_url: String,      // e.g. https://quote-api.jup.ag/v6
    pub slippage_bps: u16,     // 50 = 0.5%
}
impl JupiterHttp {
    pub fn new(base_url: impl Into<String>, slippage_bps: u16) -> Self {
        let client = reqwest::Client::builder()
            .use_rustls_tls()
            .timeout(std::time::Duration::from_secs(5))
            .build().unwrap();
        Self { client, base_url: base_url.into(), slippage_bps }
    }
}

/* ---------- HTTP 层 trait（你自己的） ---------- */
#[derive(Debug, Clone)]
pub struct QuoteReq { pub input_mint: String, pub output_mint: String, pub amount: u64 }

#[derive(Debug, Clone)]
pub struct QuoteResp { pub out_amount: u64, pub label: String }

#[async_trait]
pub trait HttpQuoteProvider: Send + Sync {
    async fn quote(&self, req: QuoteReq) -> Result<QuoteResp>;
}

#[async_trait]
impl HttpQuoteProvider for JupiterHttp {
    async fn quote(&self, req: QuoteReq) -> Result<QuoteResp> {
        // 复用你写的 fetch_jupiter_quote（返回 QuoteInfo，含字符串 out_amount）
        let QuoteInfo { out_amount, label } = fetch_jupiter_quote(
            &req.input_mint, &req.output_mint, req.amount, self.slippage_bps
        ).await?;

        let out = out_amount.parse::<u64>()
            .map_err(|_| anyhow!("bad outAmount: {}", out_amount))?;

        Ok(QuoteResp { out_amount: out, label })
    }
}

/* ---------- 策略层 trait（适配为 u64） ---------- */
#[allow(async_fn_in_trait)]
impl StratQuoteProvider for JupiterHttp {
    async fn quote(&self, input: String, output: String, amount: u64) -> Result<u64> {
        let r = QuoteReq { input_mint: input, output_mint: output, amount };
        Ok(<JupiterHttp as HttpQuoteProvider>::quote(self, r).await?.out_amount)
    }
}

/* 可选：保留旧路径导出 */
pub mod api {
    pub use super::{JupiterHttp, HttpQuoteProvider, QuoteReq, QuoteResp};
}
