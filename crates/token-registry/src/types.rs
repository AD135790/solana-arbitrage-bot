use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub symbol: String,     // 建议统一大写存
    pub mint: String,
    pub decimals: u8,
    #[serde(default)]
    pub aliases: Vec<String>,
}

impl TokenInfo {
    /// 产出所有可用 key（大写 symbol/alias）
    pub fn keys(&self) -> Vec<String> {
        let mut v = Vec::with_capacity(1 + self.aliases.len());
        v.push(self.symbol.to_ascii_uppercase());
        for a in &self.aliases {
            v.push(a.to_ascii_uppercase());
        }
        v
    }
}
