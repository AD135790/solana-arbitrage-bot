use token_registry::{LocalResolver, JupiterV2, Registry, MintResolver};
use token_registry::types::TokenInfo;

// ---------- 本地解析 + 别名 + 大小写 ----------
#[test]
fn local_resolver_basic_and_alias() {
    let local = LocalResolver::with_builtin();

    // 基础
    let sol_mint = local.get_mint("SOL").unwrap();
    assert_eq!(sol_mint, "So11111111111111111111111111111111111111112");
    assert_eq!(local.get_decimals("SOL"), Some(9));

    // 别名（WSOL / wsol）
    let sol_mint_alias = local.get_mint("wsol").unwrap();
    assert_eq!(sol_mint_alias, sol_mint);

    // USDC
    assert_eq!(
        local.get_mint("usdc").unwrap(),
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
    );
    assert_eq!(local.get_decimals("USDC"), Some(6));
}

// ---------- Registry 远端回退 ----------
#[test]
fn registry_fallback_to_remote() {
    let local = LocalResolver::with_builtin();

    // 远端（这里用构造好的列表模拟 Jupiter v2）
    let remote_list = vec![TokenInfo {
        symbol: "BONK".into(),
        mint: "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".into(),
        decimals: 5,
        aliases: vec![],
    }];
    let remote = JupiterV2::from_list(remote_list);

    let reg = Registry::with_remote(local, remote);

    // 本地没有 BONK，应该从远端返回
    assert_eq!(
        reg.get_mint("bonk").unwrap(),
        "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263"
    );
    assert_eq!(reg.get_decimals("BONK"), Some(5));
}

// ---------- 未知符号应报错 ----------
#[test]
fn unknown_symbol_errors() {
    let local = LocalResolver::with_builtin();
    let reg = Registry::local_only(local);

    let err = reg.get_mint("UNKNOWN").unwrap_err().to_string();
    assert!(err.to_lowercase().contains("unknown"), "err={}", err);
}

// ---------- tradable 透传（当前实现恒为 true） ----------
#[test]
fn tradable_passthrough() {
    let local = LocalResolver::with_builtin();
    let reg = Registry::local_only(local);
    let sol = reg.get_mint("SOL").unwrap();
    assert_eq!(reg.is_tradable(sol), Some(true));
}
