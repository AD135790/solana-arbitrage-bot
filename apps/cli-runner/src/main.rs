mod commands;
mod handlers;

use clap::{Parser, Subcommand};
use commands::quote::{run as run_quote, QuoteArgs};
use commands::quote_matrix::{QuoteMatrixArgs};       // ✅ 加
use handlers::quote_matrix::handle_quote_matrix; 

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
    QuoteMatrix(QuoteMatrixArgs),
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("❌ CLI 执行出错: {e}");
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    use clap::Parser;

    let cli = Cli::parse();

    match cli.command {
        Commands::Quote(args) => run_quote(args).await?,
        Commands::QuoteMatrix(args) => handle_quote_matrix(args).await?,
    }

    Ok(())
}
