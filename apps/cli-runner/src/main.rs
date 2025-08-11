mod commands;
mod handlers;
mod clients;

use clap::{Parser, Subcommand};
use commands::quote::{run as run_quote, QuoteArgs};
use commands::quote_matrix::QuoteMatrixArgs;
use handlers::quote_matrix::handle_quote_matrix;
use crate::clients::{build_provider, ProviderKind};

use token_registry::LocalMintResolver; // 如果你没有这个类型，就用 Registry::from_sources(...)

#[derive(Parser)]
#[command(name = "cli-runner")]
#[command(about = "🧪 CLI 工具：本地测试套利路径和数据链路", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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
    let cli = Cli::parse();

    match cli.command {
        Commands::Quote(args) => run_quote(args).await?,
        Commands::QuoteMatrix(args) => {
            // 1) resolver
            let resolver = LocalMintResolver::new(); // 或 Registry::from_sources(...)
            let require_tradable = true;

            // 2) quoter（用工厂：Jupiter/Mock 可切换）
            let quoter = build_provider(ProviderKind::Jupiter, args.qps, args.retries);

            // 3) 跑
            handle_quote_matrix(args, &resolver, &quoter, require_tradable).await?;
        }
    }

    Ok(())
}
