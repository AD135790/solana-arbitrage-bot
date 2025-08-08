use client::jupiter::quote::fetch_jupiter_quote;
use utils::resolve_mint_address;
use client::jupiter::arbitrage::fetch_jupiter_routes;
use crate::commands::quote::QuoteArgs;
use anyhow::{anyhow, Result};

pub async fn handle_quote(args: QuoteArgs) -> Result<()> {
    let input_mint = resolve_mint_address(&args.input)
        .ok_or_else(|| anyhow!("❌ 无效的 input token"))?;

    let output_mint = resolve_mint_address(&args.output)
        .ok_or_else(|| anyhow!("❌ 无效的 output token"))?;

    let lamports = (args.amount * 1_000_000_000.0) as u64;

    // ✅ quote 获取 + 容错处理
    let quote = match fetch_jupiter_quote(input_mint, output_mint, lamports).await {
        Ok(q) => q,
        Err(e) => {
            eprintln!("❌ quote 获取失败: {} → {} | 错误: {}", args.input, args.output, e);
            return Ok(()); // 🧼 不 panic，返回空
        }
    };

    println!("\n🔍 Quote Result ({} → {}):", args.input, args.output);
    println!("- Estimated Output: {}", quote.out_amount);
    println!("- First Route Label: {}", quote.label);

    // ✅ 如果 verbose，打印详细路线
    if args.verbose {
        println!("\n📦 All routePlan entries:");

        match fetch_jupiter_routes(input_mint, output_mint, lamports).await {
            Ok(all_routes) => {
                for (i, route) in all_routes.iter().enumerate() {
                    println!(
                        "{}. label: {:<12} out: {} hops: {}",
                        i + 1,
                        route.label,
                        route.out_amount,
                        route.hops
                    );
                }
            }
            Err(e) => {
                eprintln!("⚠️ 获取全部路线失败: {}", e);
            }
        }
    }

    Ok(())
}
