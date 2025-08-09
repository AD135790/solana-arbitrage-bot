use anyhow::{Result, bail};
use crate::{api::MintResolver, types::TokenInfo};
use crate::registry::local_tokens; // 你之前的 local_tokens() 放哪就从哪引

/// 基于本地表的 Mint 解析器
pub struct LocalMintResolver {
    tokens: Vec<TokenInfo>,
}

impl LocalMintResolver {
    pub fn new() -> Self {
        Self { tokens: local_tokens() }
    }

    fn find_by_symbol_or_alias<'a>(&'a self, s: &str) -> Option<&'a TokenInfo> {
        let s = s.trim();
        self.tokens.iter().find(|t|
            t.symbol.eq_ignore_ascii_case(s) ||
            t.aliases.iter().any(|a| a.eq_ignore_ascii_case(s))
        )
    }
}

impl Default for LocalMintResolver {
    fn default() -> Self { Self::new() }
}

impl MintResolver for LocalMintResolver {
    // ✅ 返回 &str（借用自 self.tokens）
    fn get_mint(&self, sym_or_mint: &str) -> Result<&str> {
        let s = sym_or_mint.trim();

        // 1) 先按 symbol/alias（不区分大小写）
        if let Some(t) = self.find_by_symbol_or_alias(s) {
            return Ok(t.mint.as_str());
        }

        // 2) 再按 “就是 mint 本身”（区分大小写）
        if let Some(t) = self.tokens.iter().find(|t| t.mint == s) {
            return Ok(t.mint.as_str());
        }

        bail!("未知符号: {}", sym_or_mint)
    }

    fn is_tradable(&self, mint: &str) -> Option<bool> {
        Some(self.tokens.iter().any(|t| t.mint == mint))
    }

    fn get_decimals(&self, sym_or_mint: &str) -> Option<u8> {
        let s = sym_or_mint.trim();
        self.find_by_symbol_or_alias(s)
            .map(|t| t.decimals)
            .or_else(|| self.tokens.iter().find(|t| t.mint == s).map(|t| t.decimals))
    }
}



