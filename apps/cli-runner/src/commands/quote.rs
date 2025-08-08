use crate::handlers::quote::handle_quote;
use clap::Args;

#[derive(Args)]
pub struct QuoteArgs {
    pub input: String,                    // 例：SOL
    pub output: String,                   // 例：USDC
    #[arg(long, default_value = "1.0")]
    pub amount: f64,
    #[arg(long, default_value = "50")]
    pub slippage: u16,
    #[arg(long, default_value_t = false)]
    pub verbose: bool,
}

pub async fn run(args: QuoteArgs) -> anyhow::Result<()> {
    handle_quote(args).await
}
