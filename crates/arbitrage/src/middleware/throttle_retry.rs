use std::time::Duration;
use tokio::time::sleep;
use anyhow::Result;
use crate::ports::quote::QuoteProvider;

pub struct ThrottleRetry {
    pub qps: u32,
    pub retries: u32,
}

pub struct Wrapped<'a, Q: QuoteProvider + ?Sized> {
    inner: &'a Q,
    interval_ms: u64,
    retries: u32,
}

impl ThrottleRetry {
    pub fn wrap<'a, Q: QuoteProvider + ?Sized>(&'a self, inner: &'a Q) -> Wrapped<'a, Q> {
        let interval_ms = (1000f64 / self.qps.max(1) as f64) as u64;
        Wrapped { inner, interval_ms, retries: self.retries }
    }
}

impl<'a, Q: QuoteProvider + ?Sized + Sync> QuoteProvider for Wrapped<'a, Q> {
    async fn quote(&self, im: String, om: String, amount: u64) -> Result<u64> {
        let mut attempt = 0u32;
        let mut backoff = 100u64;
        loop {
            match self.inner.quote(im.clone(), om.clone(), amount).await {
                Ok(x) => { sleep(Duration::from_millis(self.interval_ms)).await; return Ok(x); }
                Err(e) => {
                    attempt += 1;
                    if attempt > self.retries { return Err(e); }
                    sleep(Duration::from_millis(backoff)).await;
                    backoff = (backoff * 2).min(1200);
                }
            }
        }
    }
}
