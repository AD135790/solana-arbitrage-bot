mod commands;
mod handlers;
mod jupiter_client;

use clap::{Parser, Subcommand};
use commands::quote::{run as run_quote, QuoteArgs};
use commands::quote_matrix::QuoteMatrixArgs;
use handlers::quote_matrix::handle_quote_matrix;

use token_registry::LocalMintResolver;
use crate::jupiter_client::api::JupiterHttp;

#[derive(Parser)]
#[command(name = "cli-runner")]
#[command(about = "🧪 CLI 工具：本地测试套利路径和数据链路", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 查询 Jupiter 报价
    Quote(QuoteArgs),
    /// 路径矩阵评估（base + 多个 tokens）
    QuoteMatrix(QuoteMatrixArgs),
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("❌ CLI 执行出错: {e}");
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Quote(args) => run_quote(args).await?,
        Commands::QuoteMatrix(args) => {
            let resolver = LocalMintResolver::new();
            let quoter   = JupiterHttp::new(args.slippage_bps); // ✅ 用 CLI 滑点
            let require_tradable = true;

            handle_quote_matrix(args, &resolver, &quoter, require_tradable).await?;
        }
    }

    Ok(())
}
