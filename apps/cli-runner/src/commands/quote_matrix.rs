use clap::Args;

/// quote-matrix 子命令参数
#[derive(Debug, Args)]
pub struct QuoteMatrixArgs {
    /// 起始币种
    pub base: String,

    /// 目标币种列表（可多个）
    pub tokens: Vec<String>,
    
    /// 显示详细路径收益信息
    #[arg(long)]
    pub verbose: bool,
}
