use anyhow::{Result, anyhow};
use crate::api::MintResolver;

/* 标记类型：无远端 */
pub struct NoRemote;

/* 默认 R=NoRemote */
pub struct Registry<L, R = NoRemote> {
    local: L,
    remote: Option<R>,
}

/* 仅本地构造器：返回 Registry<L, NoRemote>，类型明确 */
impl<L> Registry<L, NoRemote> {
    pub fn local_only(local: L) -> Self {
        Self { local, remote: None }
    }
}

/* 有远端的构造器 */
impl<L, R> Registry<L, R> {
    pub fn with_remote(local: L, remote: R) -> Self {
        Self { local, remote: Some(remote) }
    }
}

/* ---- 只有本地的实现：专门针对 NoRemote ---- */
impl<L: MintResolver> MintResolver for Registry<L, NoRemote> {
    fn get_mint(&self, symbol: &str) -> Result<&str> {
        self.local.get_mint(symbol)
    }
    fn get_decimals(&self, s: &str) -> Option<u8> {
        self.local.get_decimals(s)
    }
    fn is_tradable(&self, mint: &str) -> Option<bool> {
        self.local.is_tradable(mint)
    }
}

/* ---- 有远端回退的实现 ---- */
impl<L: MintResolver, R: MintResolver> MintResolver for Registry<L, R> {
    fn get_mint(&self, symbol: &str) -> Result<&str> {
        self.local
            .get_mint(symbol)
            .or_else(|_| match &self.remote {
                Some(r) => r.get_mint(symbol),
                None => Err(anyhow!("unknown symbol: {symbol}")),
            })
    }
    fn get_decimals(&self, s: &str) -> Option<u8> {
        self.local
            .get_decimals(s)
            .or_else(|| self.remote.as_ref().and_then(|r| r.get_decimals(s)))
    }
    fn is_tradable(&self, mint: &str) -> Option<bool> {
        self.local
            .is_tradable(mint)
            .or_else(|| self.remote.as_ref().and_then(|r| r.is_tradable(mint)))
    }
}
