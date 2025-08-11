pub mod jupiter_http;
pub mod mock;
// 将来: pub mod orca_http;

use anyhow::Result;
use arbitrage::QuoteProvider as StratQuoter;

// 供 CLI 选择
#[derive(Clone, Copy, Debug)]
pub enum ProviderKind {
    Jupiter,
    Mock,
    // Orca,
}

// 统一的“具体类型”，避免 Box<dyn …>
pub enum Provider {
    // 拥有型：限速+重试 包装 真实客户端
    Jupiter(jupiter_http::ThrottleRetry<jupiter_http::JupiterHttp>),
    Mock(mock::MockQuoter),
    // Orca(...),
}

impl Provider {
    pub fn build(kind: ProviderKind, qps: u32, retries: u32) -> Self {
        match kind {
            ProviderKind::Jupiter => {
                let inner = jupiter_http::JupiterHttp::new();
                let throttled = jupiter_http::ThrottleRetry::new(inner, qps, retries);
                Provider::Jupiter(throttled)
            }
            ProviderKind::Mock => Provider::Mock(mock::MockQuoter),
            // ProviderKind::Orca => { ... }
        }
    }
}

// 让枚举充当报价器（模式匹配转发）
impl StratQuoter for Provider {
    async fn quote(&self, input_mint: String, output_mint: String, amount: u64) -> Result<u64> {
        match self {
            Provider::Jupiter(q) => q.quote(input_mint, output_mint, amount).await,
            Provider::Mock(q)    => q.quote(input_mint, output_mint, amount).await,
            // Provider::Orca(q) => q.quote(input_mint, output_mint, amount).await,
        }
    }
}

// 如果你更喜欢保持原来的函数名：
pub fn build_provider(kind: ProviderKind, qps: u32, retries: u32) -> Provider {
    Provider::build(kind, qps, retries)
}
