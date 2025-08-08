use crate::commands::quote_matrix::QuoteMatrixArgs;
use client::jupiter::quote_chain::fetch_chain_quotes;
// use client::jupiter::token_list::fetch_supported_tokens; // ❌ 暂时不用，网络不通
use utils::printer::{MatrixRow, print_matrix_table};
use utils::resolve_mint_address;

pub async fn handle_quote_matrix(args: QuoteMatrixArgs) -> Result<(), Box<dyn std::error::Error>> {
    let base = args.base.to_uppercase();
    let tokens = args.tokens.iter().map(|s| s.to_uppercase()).collect::<Vec<_>>();
    let mut result_rows: Vec<MatrixRow> = vec![];

    println!("🧪 开始构造套利路径（起点：{}）", base);

    // ❌ 暂时不用网络请求的 token list
    // let supported_tokens = fetch_supported_tokens().await?;

    for token in &tokens {
        let mid = token;
        let path = vec![base.as_str(), mid.as_str(), base.as_str()];
        let start_amount = 1_000_000_000u64; // 1 SOL

        // ✅ 只通过 TOKEN_MAP 映射解析
        let _base_mint = match resolve_mint_address(&base) {
            Some(m) => m,
            None => {
                println!("⚠️ 无法识别 base 币种: {}", base);
                continue;
            }
        };

        let _mid_mint = match resolve_mint_address(mid) {
            Some(m) => m,
            None => {
                println!("⚠️ 无法识别中间币种: {}", mid);
                continue;
            }
        };

        // ❌ 如果你未来网络恢复，可打开以下检查
        // if !supported_tokens.contains(base_mint) {
        //     println!("⚠️ Jupiter 不支持 base 币种: {}", base);
        //     continue;
        // }
        // if !supported_tokens.contains(mid_mint) {
        //     println!("⚠️ Jupiter 不支持中间币种: {}", mid);
        //     continue;
        // }

        // ✅ 执行模拟路径报价
        match fetch_chain_quotes(path, start_amount).await {
            Ok(chain) => {
                let final_sol = chain.final_amount as f64 / 1e9;
                let profitable = chain.final_amount > start_amount;

                result_rows.push(MatrixRow {
                    profitable,
                    path: format!("{} → {} → {}", base, mid, base),
                    start: 1.0,
                    end: final_sol,
                });
            }
            Err(e) => {
                println!("❌ 无法完成路径 {} → {} → {} | 错误: {}", base, mid, base, e);
                continue;
            }
        }
    }

    // ✅ 打印结果表格
    print_matrix_table(result_rows);
    Ok(())
}
