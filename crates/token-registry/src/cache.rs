use crate::api::MintResolver;
use anyhow::Result;

/// 先放简单透传，以后要 TTL/链上校验再加
pub struct Cached<R: MintResolver> { inner: R }
impl<R: MintResolver> Cached<R> { pub fn new(inner: R) -> Self { Self { inner } } }

impl<R: MintResolver> MintResolver for Cached<R> {
    fn get_mint(&self, s: &str) -> Result<&str> { self.inner.get_mint(s) }
    fn get_decimals(&self, s: &str) -> Option<u8> { self.inner.get_decimals(s) }
    fn is_tradable(&self, m: &str) -> Option<bool> { self.inner.is_tradable(m) }
}
