use anyhow::Result;

/// 统一抽象：策略层只依赖这个
pub trait MintResolver: Send + Sync {
    /// 输入 symbol（不区分大小写），返回 mint 地址（借用）
    fn get_mint(&self, symbol: &str) -> Result<&str>;
    /// 小数位（拿不到就 None）
    fn get_decimals(&self, symbol: &str) -> Option<u8>;
    /// 是否可交易（可用来过滤弃用/黑名单）
    fn is_tradable(&self, mint: &str) -> Option<bool>;
}
