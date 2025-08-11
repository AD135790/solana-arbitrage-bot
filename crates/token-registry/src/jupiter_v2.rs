use std::collections::HashMap;
use anyhow::{Result, anyhow};
use crate::api::MintResolver;
use crate::types::TokenInfo;

pub struct JupiterV2 {
    sym_to_mint: HashMap<String, String>,
    decimals:    HashMap<String, u8>,
}

impl JupiterV2 {
    pub fn from_list(list: Vec<TokenInfo>) -> Self {
        let mut sym_to_mint = HashMap::new();
        let mut decimals    = HashMap::new();
        for t in list {
            for k in t.keys() {
                sym_to_mint.insert(k, t.mint.clone());
            }
            decimals.insert(t.symbol.to_ascii_uppercase(), t.decimals);
        }
        Self { sym_to_mint, decimals }
    }
}

impl MintResolver for JupiterV2 {
    fn get_mint(&self, symbol: &str) -> Result<&str> {
        self.sym_to_mint
            .get(&symbol.to_ascii_uppercase())
            .map(|s| s.as_str())
            .ok_or_else(|| anyhow!("unknown symbol: {symbol}"))
    }
    fn get_decimals(&self, symbol: &str) -> Option<u8> {
        self.decimals.get(&symbol.to_ascii_uppercase()).copied()
    }
    fn is_tradable(&self, _mint: &str) -> Option<bool> { Some(true) }
}
