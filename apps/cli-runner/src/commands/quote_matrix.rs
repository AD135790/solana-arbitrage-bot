use clap::{Args, ValueEnum};
use std::path::PathBuf;

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum Hops { Two, Three }

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum ExportFormat { Csv, Json }

#[derive(Debug, Args, Clone)]
pub struct QuoteMatrixArgs {
    /// Base token symbol or mint (e.g. SOL)
    pub base: String,

    /// Mid tokens to scan (symbols or mints)
    #[arg(num_args = 1.., value_name = "TOKENS")]
    pub tokens: Vec<String>,

    // ---- 金额与跳数 ----
    /// Amount in human units of BASE (e.g. 1.0 SOL). 与 --amount-lamports 互斥
    #[arg(long, default_value_t = 1.0, conflicts_with = "amount_lamports")]
    pub amount: f64,

    /// Amount in smallest units (e.g. lamports). 与 --amount 互斥
    #[arg(long)]
    pub amount_lamports: Option<u64>,

    /// 路径跳数（Two=BASE→X→BASE，Three=BASE→X→Y→BASE）
    #[arg(long, value_enum, default_value_t = Hops::Two)]
    pub hops: Hops,

    // ---- 过滤与排序 ----
    /// 最小“毛收益”阈值（百分比，0.10 = 0.10%），低于此不展示
    #[arg(long, default_value_t = 0.10)]
    pub min_change: f64,

    /// 最小“净收益”阈值（扣费+滑点后，百分比），用于告警/执行
    #[arg(long, default_value_t = 0.30)]
    pub min_net_change: f64,

    /// 只展示 Top-K（0 表示全量）
    #[arg(long, default_value_t = 0)]
    pub top_k: usize,

    /// 仅扫描在 Jupiter “可交易集合”中的代币
    #[arg(long, default_value_t = true)]
    pub require_tradable: bool,

    // ---- 费用/滑点建模 ----
    /// 每跳手续费（bps），0.25% = 25
    #[arg(long, default_value_t = 25)]
    pub fee_bps_per_hop: u16,

    /// 每跳假设滑点（bps）
    #[arg(long, default_value_t = 30)]
    pub slippage_bps: u16,

    // ---- 性能与稳定性 ----
    /// 最大并发请求数
    #[arg(long, default_value_t = 5)]
    pub concurrency: usize,

    /// 速率限制（每秒最多请求数）
    #[arg(long, default_value_t = 8)]
    pub qps: u32,

    /// 每次报价最大重试次数
    #[arg(long, default_value_t = 3)]
    pub retries: u32,

    // ---- 导出 ----
    /// 导出路径（不填则不导出）
    #[arg(long)]
    pub export: Option<PathBuf>,

    /// 导出格式（csv/json）
    #[arg(long, value_enum, default_value_t = ExportFormat::Csv)]
    pub export_format: ExportFormat,

    // ---- 监听与告警 ----
    /// 关注列表文件（每行一个 symbol/mint）
    #[arg(long)]
    pub watch_list: Option<PathBuf>,

    /// Telegram 机器人 token（用于告警）
    #[arg(long)]
    pub tg_token: Option<String>,

    /// Telegram chat id（字符串或数字）
    #[arg(long)]
    pub tg_chat: Option<String>,

    // ---- 输出 ----
    /// 打印更多 route 信息
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    #[arg(long, default_value_t = false)]
    pub json: bool,

    #[arg(long)]
    pub meme: bool,

    
}
