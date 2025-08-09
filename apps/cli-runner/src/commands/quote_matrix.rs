use clap::{Args, ValueEnum};

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum Hops { 
    Two,
    Three,
    }

#[derive(Debug, Args, Clone)]
pub struct QuoteMatrixArgs {
    pub base: String,
    #[arg(num_args = 1.., value_name = "TOKENS")]
    pub tokens: Vec<String>,

    #[arg(long, default_value_t = 1.0)]
    pub amount: f64,
    #[arg(long, default_value_t = 50)]
    pub slippage_bps: u16,
    #[arg(long, default_value_t = 5)]
    pub concurrency: usize,

    #[arg(long, value_enum, default_value_t = Hops::Two)]
    pub hops: Hops,

    /// 只展示 Top-K（0 表示全量）
    #[arg(long, default_value_t = 0)]
    pub top_k: usize,

    #[arg(long, default_value_t = false)]
    pub json: bool,
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
}
