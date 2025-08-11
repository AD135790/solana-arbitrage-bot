use std::collections::HashMap;
use anyhow::{Result, anyhow};
use crate::api::MintResolver;
use crate::types::TokenInfo;

pub struct LocalResolver {
    sym_to_mint: HashMap<String, String>, // 大写 symbol/alias -> mint
    decimals:    HashMap<String, u8>,     // 大写 symbol -> decimals
    tradable:    HashMap<String, bool>,   // mint -> tradable
}

impl LocalResolver {
    pub fn from_tokens(tokens: Vec<TokenInfo>) -> Self {
        let mut sym_to_mint = HashMap::new();
        let mut decimals    = HashMap::new();
        for t in tokens {
            for k in t.keys() {
                sym_to_mint.insert(k, t.mint.clone());
            }
            decimals.insert(t.symbol.to_ascii_uppercase(), t.decimals);
        }
        Self { sym_to_mint, decimals, tradable: HashMap::new() }
    }

    pub fn with_builtin() -> Self {
        use TokenInfo as T;
        let toks = vec![
            T { symbol: "SOL".into(),  mint: "So11111111111111111111111111111111111111112".into(), decimals: 9, aliases: vec!["WSOL".into(), "wSOL".into()] },
            T { symbol: "USDC".into(), mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".into(), decimals: 6, aliases: vec![] },
            T { symbol: "MSOL".into(), mint: "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So".into(), decimals: 9, aliases: vec!["mSOL".into()] },
        ];
        Self::from_tokens(toks)
    }
}

impl MintResolver for LocalResolver {
    fn get_mint(&self, symbol: &str) -> Result<&str> {
        let key = symbol.to_ascii_uppercase();
        self.sym_to_mint
            .get(&key)
            .map(|s| s.as_str())
            .ok_or_else(|| anyhow!("unknown symbol: {symbol}"))
    }
    fn get_decimals(&self, symbol: &str) -> Option<u8> {
        self.decimals.get(&symbol.to_ascii_uppercase()).copied()
    }
    fn is_tradable(&self, _mint: &str) -> Option<bool> { Some(true) }
}
