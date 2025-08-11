use anyhow::Result;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::time::Duration;

use crate::commands::quote_matrix::{QuoteMatrixArgs, Hops, ExportFormat};
use arbitrage::{evaluate_2hop, evaluate_3hop, ArbRow, QuoteProvider as StratQuoter};
use token_registry::api::MintResolver;

use utils::printer::{MatrixRow, print_matrix_table};

use serde::Serialize;
use tokio::time::sleep;

// ===================== 限速 + 重试包装器（针对任意 Q） =====================
pub struct ThrottleRetry<'a, Q: StratQuoter + ?Sized> {
    inner: &'a Q,
    interval_ms: u64,
    retries: u32,
}

impl<'a, Q: StratQuoter + ?Sized> ThrottleRetry<'a, Q> {
    pub fn new(inner: &'a Q, qps: u32, retries: u32) -> Self {
        let interval_ms = (1000f64 / qps.max(1) as f64) as u64;
        Self { inner, interval_ms, retries }
    }
}

impl<'a, Q: StratQuoter + Send + Sync + ?Sized> StratQuoter for ThrottleRetry<'a, Q> {
    async fn quote(&self, input_mint: String, output_mint: String, amount: u64) -> Result<u64> {
        let mut attempt = 0u32;
        let mut backoff = 100u64;
        loop {
            match self.inner.quote(input_mint.clone(), output_mint.clone(), amount).await {
                Ok(out) => {
                    sleep(Duration::from_millis(self.interval_ms)).await; // 限速
                    return Ok(out);
                }
                Err(e) => {
                    attempt += 1;
                    if attempt > self.retries {
                        return Err(e);
                    }
                    sleep(Duration::from_millis(backoff)).await;          // 退避
                    backoff = (backoff * 2).min(1200);
                }
            }
        }
    }
}

// ===================== 导出与告警辅助 =====================
#[derive(Serialize)]
struct RowOut {
    path: String,
    start: f64,
    end_gross: f64,
    end_net: f64,
    gross_pct: f64,
    net_pct: f64,
    profitable_net: bool,
    hops: u32,
    ts: i64,
}

async fn send_telegram(token: &str, chat_id: &str, text: &str) -> Result<()> {
    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
    let body = serde_json::json!({ "chat_id": chat_id, "text": text });
    let client = reqwest::Client::new();
    let _ = client.post(url).json(&body).send().await?;
    Ok(())
}

fn unix_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

// ===================== 主处理函数（泛型 Q） =====================
pub async fn handle_quote_matrix<R, Q>(
    args: QuoteMatrixArgs,
    resolver: &R,
    quoter: &Q,                 // ✅ 任何实现了 StratQuoter 的类型
    require_tradable: bool,
) -> Result<()>
where
    R: MintResolver,
    Q: StratQuoter + Sync,
{
    
    // 三跳时，默认并发自动降到 2 更稳
    let effective_conc = match args.hops {
        Hops::Three if args.concurrency == 5 => 2,
        _ => args.concurrency,
    };

    let hops_u32 = match args.hops { Hops::Two => 2, Hops::Three => 3 };
    let fee_slip_per_hop_bps = args.fee_bps_per_hop as i32 + args.slippage_bps as i32;
    let total_fee_slip_bps = fee_slip_per_hop_bps * hops_u32 as i32;

    println!(
        "🧪 扫描 | base={} amount={} hops={:?} fee_per_hop={}bps slip={}bps total_fee_slip={}bps conc={} qps={} retries={}",
        args.base, args.amount, args.hops, args.fee_bps_per_hop, args.slippage_bps,
        total_fee_slip_bps, effective_conc, args.qps, args.retries
    );

    // 包一层限速+重试（不转移所有权，包的是 &Q）
    let throttled = ThrottleRetry::new(quoter, args.qps, args.retries);

    // 可选：在扫描前做 tradable 过滤
    let tokens_filtered: Vec<String> = if require_tradable {
        args.tokens
            .iter()
            .cloned()
            .filter(|sym| {
                if let Ok(mint) = resolver.get_mint(sym) {
                    resolver.is_tradable(mint).unwrap_or(true)
                } else { false }
            })
            .collect()
    } else {
        args.tokens.clone()
    };

    // 1) 计算（策略层不做最小盈利阈值）
    let mut rows: Vec<ArbRow> = match args.hops {
        Hops::Two => {
            evaluate_2hop(
                resolver, &throttled, &args.base, &tokens_filtered, args.amount, i32::MIN, effective_conc
            ).await
        }
        Hops::Three => {
            let r = evaluate_3hop(
                resolver, &throttled, &args.base, &tokens_filtered, args.amount, effective_conc, args.verbose
            ).await;
            if r.is_empty() {
                eprintln!("ℹ️ 3-hop 返回为空，回退跑一轮 2-hop…");
                evaluate_2hop(
                    resolver, &throttled, &args.base, &tokens_filtered, args.amount, i32::MIN, effective_conc
                ).await
            } else { r }
        }
    };use anyhow::Result;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::time::Duration;

use crate::commands::quote_matrix::{QuoteMatrixArgs, Hops, ExportFormat};
use arbitrage::{evaluate_2hop, evaluate_3hop, ArbRow, QuoteProvider as StratQuoter};
use token_registry::api::MintResolver;

use utils::printer::{MatrixRow, print_matrix_table};

use serde::Serialize;
use tokio::time::sleep;

// ===================== 限速 + 重试包装器（针对任意 Q） =====================
pub struct ThrottleRetry<'a, Q: StratQuoter + ?Sized> {
    inner: &'a Q,
    interval_ms: u64,
    retries: u32,
}

impl<'a, Q: StratQuoter + ?Sized> ThrottleRetry<'a, Q> {
    pub fn new(inner: &'a Q, qps: u32, retries: u32) -> Self {
        let interval_ms = (1000f64 / qps.max(1) as f64) as u64;
        Self { inner, interval_ms, retries }
    }
}

impl<'a, Q: StratQuoter + Send + Sync + ?Sized> StratQuoter for ThrottleRetry<'a, Q> {
    async fn quote(&self, input_mint: String, output_mint: String, amount: u64) -> Result<u64> {
        let mut attempt = 0u32;
        let mut backoff = 100u64;
        loop {
            match self.inner.quote(input_mint.clone(), output_mint.clone(), amount).await {
                Ok(out) => {
                    sleep(Duration::from_millis(self.interval_ms)).await; // 限速
                    return Ok(out);
                }
                Err(e) => {
                    attempt += 1;
                    if attempt > self.retries {
                        return Err(e);
                    }
                    sleep(Duration::from_millis(backoff)).await;          // 退避
                    backoff = (backoff * 2).min(1200);
                }
            }
        }
    }
}

// ===================== 导出与告警辅助 =====================
#[derive(Serialize)]
struct RowOut {
    path: String,
    start: f64,
    end_gross: f64,
    end_net: f64,
    gross_pct: f64,
    net_pct: f64,
    profitable_net: bool,
    hops: u32,
    ts: i64,
}

async fn send_telegram(token: &str, chat_id: &str, text: &str) -> Result<()> {
    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
    let body = serde_json::json!({ "chat_id": chat_id, "text": text });
    let client = reqwest::Client::new();
    let _ = client.post(url).json(&body).send().await?;
    Ok(())
}

fn unix_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

// ===================== 主处理函数（泛型 Q） =====================
pub async fn handle_quote_matrix<R, Q>(
    args: QuoteMatrixArgs,
    resolver: &R,
    quoter: &Q,                 // ✅ 任何实现了 StratQuoter 的类型
    require_tradable: bool,
) -> Result<()>
where
    R: MintResolver,
    Q: StratQuoter + Sync,
{
    
    // 三跳时，默认并发自动降到 2 更稳
    let effective_conc = match args.hops {
        Hops::Three if args.concurrency == 5 => 2,
        _ => args.concurrency,
    };

    let hops_u32 = match args.hops { Hops::Two => 2, Hops::Three => 3 };
    let fee_slip_per_hop_bps = args.fee_bps_per_hop as i32 + args.slippage_bps as i32;
    let total_fee_slip_bps = fee_slip_per_hop_bps * hops_u32 as i32;

    println!(
        "🧪 扫描 | base={} amount={} hops={:?} fee_per_hop={}bps slip={}bps total_fee_slip={}bps conc={} qps={} retries={}",
        args.base, args.amount, args.hops, args.fee_bps_per_hop, args.slippage_bps,
        total_fee_slip_bps, effective_conc, args.qps, args.retries
    );

    // 包一层限速+重试（不转移所有权，包的是 &Q）
    let throttled = ThrottleRetry::new(quoter, args.qps, args.retries);

    // 可选：在扫描前做 tradable 过滤
    let tokens_filtered: Vec<String> = if require_tradable {
        args.tokens
            .iter()
            .cloned()
            .filter(|sym| {
                if let Ok(mint) = resolver.get_mint(sym) {
                    resolver.is_tradable(mint).unwrap_or(true)
                } else { false }
            })
            .collect()
    } else {
        args.tokens.clone()
    };

    // 1) 计算（策略层不做最小盈利阈值）
    let mut rows: Vec<ArbRow> = match args.hops {
        Hops::Two => {
            evaluate_2hop(
                resolver, &throttled, &args.base, &tokens_filtered, args.amount, i32::MIN, effective_conc
            ).await
        }
        Hops::Three => {
            let r = evaluate_3hop(
                resolver, &throttled, &args.base, &tokens_filtered, args.amount, effective_conc, args.verbose
            ).await;
            if r.is_empty() {
                eprintln!("ℹ️ 3-hop 返回为空，回退跑一轮 2-hop…");
                evaluate_2hop(
                    resolver, &throttled, &args.base, &tokens_filtered, args.amount, i32::MIN, effective_conc
                ).await
            } else { r }
        }
    };

    // 2) 去重（按 path）
    let mut seen = HashSet::new();
    rows.retain(|r| seen.insert(r.path.clone()));

    // 3) 按毛收益排序（delta_bps 高在前）
    rows.sort_by(|a, b| b.delta_bps.partial_cmp(&a.delta_bps).unwrap_or(Ordering::Equal));

    // 4) 计算净收益 & 过滤/导出/告警
    let now_ts = unix_ts();
    let mut export_rows: Vec<RowOut> = Vec::new();

    // 表格视图（用净 bps 驱动 Δ）
    let mut view_rows: Vec<MatrixRow> = Vec::new();

    for r in rows.into_iter() {
        // 毛收益（bps → %）
        let gross_bps = r.delta_bps as i32;
        let gross_pct = (gross_bps as f64) / 100.0;

        // 线性净收益估算：net_bps = gross_bps - hops*(fee+slip)
        let net_bps = gross_bps - total_fee_slip_bps;
        let net_pct = (net_bps as f64) / 100.0;

        // 展示过滤：按毛收益阈值
        if gross_pct < args.min_change {
            continue;
        }

        // 估算净终值（近似：按净百分比作用于起始值）
        let end_gross = r.end;
        let end_net = r.start * (1.0 + net_pct / 100.0);

        // 表格输出：净 bps
        view_rows.push(MatrixRow {
            profitable: net_bps > 0,
            path: r.path.clone(),
            start: r.start,
            end: end_net,
            delta_bps: net_bps as f64,
        });

        // 导出结构
        export_rows.push(RowOut {
            path: r.path.clone(),
            start: r.start,
            end_gross,
            end_net,
            gross_pct,
            net_pct,
            profitable_net: net_bps > 0,
            hops: hops_u32,
            ts: now_ts,
        });

        // 告警：净收益达到阈值才提醒
        if net_pct >= args.min_net_change {
            if let (Some(tok), Some(chat)) = (&args.tg_token, &args.tg_chat) {
                let _ = send_telegram(tok, chat, &format!(
                    "✅ {}\nGross: {:.3}%  Net: {:.3}%\nEnd(net): {:.6}",
                    r.path, gross_pct, net_pct, end_net
                )).await;
            }
        }
    }

    // 5) Top-K
    if args.top_k > 0 && view_rows.len() > args.top_k {
        view_rows.truncate(args.top_k);
    }
    if args.top_k > 0 && export_rows.len() > args.top_k {
        export_rows.truncate(args.top_k);
    }

    // 6) 输出
    if args.json {
        println!("{}", serde_json::to_string_pretty(&export_rows)?);
    } else {
        print_matrix_table(view_rows);
    }

    // 7) 导出到文件
    if let Some(path) = &args.export {
        match args.export_format {
            ExportFormat::Csv => {
                let mut wtr = csv::Writer::from_path(path)?;
                for row in &export_rows { wtr.serialize(row)?; }
                wtr.flush()?;
                println!("💾 CSV 导出: {}", path.display());
            }
            ExportFormat::Json => {
                std::fs::write(path, serde_json::to_vec_pretty(&export_rows)?)?;
                println!("💾 JSON 导出: {}", path.display());
            }
        }
    }

    Ok(())
}


    // 2) 去重（按 path）
    let mut seen = HashSet::new();
    rows.retain(|r| seen.insert(r.path.clone()));

    // 3) 按毛收益排序（delta_bps 高在前）
    rows.sort_by(|a, b| b.delta_bps.partial_cmp(&a.delta_bps).unwrap_or(Ordering::Equal));

    // 4) 计算净收益 & 过滤/导出/告警
    let now_ts = unix_ts();
    let mut export_rows: Vec<RowOut> = Vec::new();

    // 表格视图（用净 bps 驱动 Δ）
    let mut view_rows: Vec<MatrixRow> = Vec::new();

    for r in rows.into_iter() {
        // 毛收益（bps → %）
        let gross_bps = r.delta_bps as i32;
        let gross_pct = (gross_bps as f64) / 100.0;

        // 线性净收益估算：net_bps = gross_bps - hops*(fee+slip)
        let net_bps = gross_bps - total_fee_slip_bps;
        let net_pct = (net_bps as f64) / 100.0;

        // 展示过滤：按毛收益阈值
        if gross_pct < args.min_change {
            continue;
        }

        // 估算净终值（近似：按净百分比作用于起始值）
        let end_gross = r.end;
        let end_net = r.start * (1.0 + net_pct / 100.0);

        // 表格输出：净 bps
        view_rows.push(MatrixRow {
            profitable: net_bps > 0,
            path: r.path.clone(),
            start: r.start,
            end: end_net,
            delta_bps: net_bps as f64,
        });

        // 导出结构
        export_rows.push(RowOut {
            path: r.path.clone(),
            start: r.start,
            end_gross,
            end_net,
            gross_pct,
            net_pct,
            profitable_net: net_bps > 0,
            hops: hops_u32,
            ts: now_ts,
        });

        // 告警：净收益达到阈值才提醒
        if net_pct >= args.min_net_change {
            if let (Some(tok), Some(chat)) = (&args.tg_token, &args.tg_chat) {
                let _ = send_telegram(tok, chat, &format!(
                    "✅ {}\nGross: {:.3}%  Net: {:.3}%\nEnd(net): {:.6}",
                    r.path, gross_pct, net_pct, end_net
                )).await;
            }
        }
    }

    // 5) Top-K
    if args.top_k > 0 && view_rows.len() > args.top_k {
        view_rows.truncate(args.top_k);
    }
    if args.top_k > 0 && export_rows.len() > args.top_k {
        export_rows.truncate(args.top_k);
    }

    // 6) 输出
    if args.json {
        println!("{}", serde_json::to_string_pretty(&export_rows)?);
    } else {
        print_matrix_table(view_rows);
    }

    // 7) 导出到文件
    if let Some(path) = &args.export {
        match args.export_format {
            ExportFormat::Csv => {
                let mut wtr = csv::Writer::from_path(path)?;
                for row in &export_rows { wtr.serialize(row)?; }
                wtr.flush()?;
                println!("💾 CSV 导出: {}", path.display());
            }
            ExportFormat::Json => {
                std::fs::write(path, serde_json::to_vec_pretty(&export_rows)?)?;
                println!("💾 JSON 导出: {}", path.display());
            }
        }
    }

    Ok(())
}
