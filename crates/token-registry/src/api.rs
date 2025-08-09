use anyhow::Result;

pub trait MintResolver: Send + Sync {
    fn get_mint(&self, sym_or_mint: &str) -> Result<&str>;
    fn get_decimals(&self, sym_or_mint: &str) -> Option<u8>;
    // 可选：是否在“可交易集合”中（不实现就返回 None）
    fn is_tradable(&self, _mint: &str) -> Option<bool> { None }
}



#[derive(Debug)]
pub struct QuoteReq<'a> {
    pub input_mint: &'a str,
    pub output_mint: &'a str,
    pub amount: u64,
    pub slippage_bps: u16,
}

#[derive(Debug)]
pub struct QuoteResp {
    pub out_amount: u64,
}

#[allow(async_fn_in_trait)]
pub trait QuoteProvider {
    async fn quote(&self, req: QuoteReq) -> anyhow::Result<QuoteResp>;
}
