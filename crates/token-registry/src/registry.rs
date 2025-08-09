use crate::TokenInfo;
use anyhow::{bail, Result};
use std::collections::{HashMap, HashSet};


#[derive(Clone, Default)]
pub struct Registry {
    by_key: HashMap<String, TokenInfo>, // 支持 symbol/alias（大写）和 mint 映射
    tradable_v1: Option<HashSet<String>>,
}

impl Registry {
    pub fn from_sources(local: Vec<TokenInfo>, v1_tradable: Option<HashSet<String>>) -> Self {
        let mut by_key = HashMap::new();
        for t in local {
            for k in t.keys() {
                by_key.insert(k, t.clone());
            }
        }
        Self { by_key, tradable_v1: v1_tradable }
    }

   pub fn get_mint(&self, sym_or_mint: &str) -> Result<&str> {
    let k = sym_or_mint.to_ascii_uppercase();

    // 1) 先按 symbol/alias（大写）找
    if let Some(t) = self.by_key.get(&k) {
        return Ok(&t.mint);
    }
    // 2) 再按“就是 mint 字符串本身”找（注意大小写敏感）
    if let Some(t) = self.by_key.get(sym_or_mint) {
        return Ok(&t.mint);
    }

    bail!("未知代币：{}（请在本地白名单补充或启用 v2 搜索）", sym_or_mint)
    }


    pub fn get_decimals(&self, sym_or_mint: &str) -> Option<u8> {
        let k = sym_or_mint.to_ascii_uppercase();
        self.by_key.get(&k).map(|t| t.decimals)
            .or_else(|| self.by_key.get(sym_or_mint).map(|t| t.decimals))
    }

    /// 基于 v1 返回的可交易集合进行快速判断（可为空）
    pub fn is_tradable_v1(&self, mint: &str) -> Option<bool> {
        self.tradable_v1.as_ref().map(|set| set.contains(mint))
    }
}

pub fn local_tokens() -> Vec<TokenInfo> {
    vec![
        TokenInfo { symbol: "SOL".into(), mint: "So11111111111111111111111111111111111111112".into(), decimals: 9, aliases: vec!["WSOL".into(), "wSOL".into()] },
        TokenInfo { symbol: "USDC".into(), mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".into(), decimals: 6, aliases: vec![] },
        TokenInfo { symbol: "mSOL".into(), mint: "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So".into(), decimals: 9, aliases: vec![] },
        TokenInfo { symbol: "JITOSOL".into(), mint: "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn".into(), decimals: 9, aliases: vec!["JitoSOL".into()] },
        TokenInfo { symbol: "BONK".into(), mint: "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".into(), decimals: 5, aliases: vec![] },
    ]
}