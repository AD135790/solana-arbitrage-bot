use anyhow::Result;
use std::cmp::Ordering;
use std::collections::HashSet;

use crate::commands::quote_matrix::{QuoteMatrixArgs, Hops};

// 策略层
use arbitrage::{evaluate_2hop, evaluate_3hop, ArbRow, QuoteProvider as StratQuoter};
use token_registry::api::MintResolver;

// HTTP 报价实现（不要引入它的 QuoteProvider，避免同名冲突）
use crate::jupiter_client::api::{JupiterHttp, QuoteReq};

// 打印
use utils::printer::{MatrixRow, print_matrix_table};

/// 让 JupiterHttp 适配策略层报价 trait
impl StratQuoter for JupiterHttp {
    async fn quote(&self, input_mint: String, output_mint: String, amount: u64) -> Result<u64> {
        // 用完全限定语法避免递归
        let resp = <JupiterHttp as crate::jupiter_client::api::QuoteProvider>::quote(
            self,
            QuoteReq { input_mint, output_mint, amount }
        ).await?;
        Ok(resp.out_amount)
    }
}

pub async fn handle_quote_matrix<R: MintResolver>(
    args: QuoteMatrixArgs,
    resolver: &R,
    quoter: &JupiterHttp,
    _require_tradable: bool,
) -> Result<()> {
    // 三跳时，如果用户没改默认并发（5），自动降到 2 更稳
    let effective_conc = match args.hops {
        Hops::Three if args.concurrency == 5 => 2,
        _ => args.concurrency,
    };

    println!(
        "🧪 开始构造套利路径（起点：{}，amount={}, slippage_bps={}，conc={}，hops={:?}）",
        args.base, args.amount, args.slippage_bps, effective_conc, args.hops
    );

    // 1) 计算（不过滤，保证全量输出）
    let mut rows: Vec<ArbRow> = match args.hops {
        Hops::Two => {
            // 传 i32::MIN，确保策略侧不做最小盈利过滤
            evaluate_2hop(
                resolver, quoter, &args.base, &args.tokens, args.amount, i32::MIN, effective_conc
            ).await
        }
        Hops::Three => {
            let r = evaluate_3hop(
                resolver, quoter, &args.base, &args.tokens, args.amount, effective_conc, args.verbose
            ).await;

            // 三跳偶发全空：自动兜底跑一轮二跳，避免空表
            if r.is_empty() {
                eprintln!("ℹ️ 3-hop returned empty. Falling back to 2-hop once…");
                evaluate_2hop(
                    resolver, quoter, &args.base, &args.tokens, args.amount, i32::MIN, effective_conc
                ).await
            } else {
                r
            }
        }
    };

    // 2) 去重（按 path）
    let mut seen = HashSet::new();
    rows.retain(|r| seen.insert(r.path.clone()));

    // 3) 排序（按 delta_bps 从高到低；NaN 视为相等）
    rows.sort_by(|a, b| b.delta_bps.partial_cmp(&a.delta_bps).unwrap_or(Ordering::Equal));

    // 4) Top-K（>0 才截断）
    if args.top_k > 0 && rows.len() > args.top_k {
        rows.truncate(args.top_k);
    }

    // 5) 输出
    if args.json {
        println!("{}", serde_json::to_string_pretty(&rows)?);
        return Ok(());
    }

    // 表格视图映射（printer 里会把 Δ 渲染成彩色百分比）
    let view_rows: Vec<MatrixRow> = rows
        .into_iter()
        .map(|r| MatrixRow {
            profitable: r.profitable,
            path: r.path,
            start: r.start,
            end: r.end,
            delta_bps: r.delta_bps,
        })
        .collect();

    print_matrix_table(view_rows);
    Ok(())
}
