use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenInfo {
    pub symbol: String,   // "SOL"
    pub mint: String,     // mint 地址
    pub decimals: u8,
    #[serde(default)]
    pub aliases: Vec<String>, // ["WSOL","wSOL"]
}

impl TokenInfo {
    pub fn keys(&self) -> Vec<String> {
        let mut ks = vec![self.symbol.to_ascii_uppercase(), self.mint.clone()];
        ks.extend(self.aliases.iter().map(|a| a.to_ascii_uppercase()));
        ks
    }
}
