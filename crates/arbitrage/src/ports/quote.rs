#![allow(async_fn_in_trait)]
use anyhow::Result;

pub trait QuoteProvider: Send + Sync {
    async fn quote(&self, input_mint: String, output_mint: String, amount: u64) -> Result<u64>;
}
