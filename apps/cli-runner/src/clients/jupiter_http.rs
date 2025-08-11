use anyhow::Result;
use arbitrage::QuoteProvider as StratQuoter;
use tokio::time::{sleep, Duration};

// 你已有的 HTTP 调用（最小实现）
pub struct JupiterHttp;

impl JupiterHttp {
    pub fn new() -> Self { Self }
    async fn raw_quote(&self, input_mint: &str, output_mint: &str, amount: u64) -> Result<u64> {
        let url = format!(
          "https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}",
          input_mint, output_mint, amount
        );
        let text = reqwest::get(&url).await?.text().await?;
        #[derive(serde::Deserialize)]
        struct Raw { #[serde(default)] outAmount: Option<String>, #[serde(default)] errorCode: Option<String> }
        let r: Raw = serde_json::from_str(&text)?;
        if let Some(code) = r.errorCode { anyhow::bail!("Jupiter: {}", code); }
        Ok(r.outAmount.ok_or_else(|| anyhow::anyhow!("missing outAmount"))?.parse()?)
    }
}

// 适配策略层
impl StratQuoter for JupiterHttp {
    async fn quote(&self, input_mint: String, output_mint: String, amount: u64) -> Result<u64> {
        self.raw_quote(&input_mint, &output_mint, amount).await
    }
}

// 可复用的限速+重试包装器（把它当最终注入对象）
pub struct ThrottleRetry<T> {
    inner: T,
    interval_ms: u64,
    retries: u32,
}
impl<T> ThrottleRetry<T> {
    pub fn new(inner: T, qps: u32, retries: u32) -> Self {
        Self { inner, interval_ms: (1000.0 / qps.max(1) as f64) as u64, retries }
    }
}
impl<T: StratQuoter + Send + Sync> StratQuoter for ThrottleRetry<T> {
    async fn quote(&self, input_mint: String, output_mint: String, amount: u64) -> Result<u64> {
        let mut tries = 0u32;
        let mut backoff = 100u64;
        loop {
            match self.inner.quote(input_mint.clone(), output_mint.clone(), amount).await {
                Ok(v) => { sleep(Duration::from_millis(self.interval_ms)).await; return Ok(v); }
                Err(e) => {
                    tries += 1;
                    if tries > self.retries { return Err(e); }
                    sleep(Duration::from_millis(backoff)).await;
                    backoff = (backoff * 2).min(1200);
                }
            }
        }
    }
}
