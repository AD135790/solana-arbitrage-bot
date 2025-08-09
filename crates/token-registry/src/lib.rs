pub mod types;
pub mod cache;
pub mod jupiter_v1;
#[cfg(feature = "v2")]
mod jupiter_v2;
pub mod registry;
pub mod api;
pub mod local_resolver;

pub use local_resolver::LocalMintResolver;
pub use types::TokenInfo;
pub use registry::Registry;
pub use api::MintResolver;  

/// 从本地白名单 + 可选的 v1 可交易集合 构建注册表
pub async fn default_registry(local: Vec<TokenInfo>) -> anyhow::Result<Registry> {
    let v1 = jupiter_v1::fetch_supported_mints().await.ok(); // 失败不致命
    Ok(Registry::from_sources(local, v1))
}

/// 仅拉取 v1 可交易 mint 集合（HashSet）
pub async fn fetch_tradable_mints_v1() -> anyhow::Result<std::collections::HashSet<String>> {
    jupiter_v1::fetch_supported_mints().await
}

/// （可选）v2 搜索：根据 symbol/name/mint 查询
#[cfg(feature = "v2")]
pub async fn search_v2(query: &str) -> anyhow::Result<Vec<TokenInfo>> {
    jupiter_v2::search(query).await
}

/// 保存/加载缓存（比如把 v1 的 mint 集合落盘）
pub fn save_tradable_cache(mints: &std::collections::HashSet<String>) -> anyhow::Result<()> {
    cache::save_mints(mints)
}
pub fn load_tradable_cache() -> Option<std::collections::HashSet<String>> {
    cache::load_mints().ok()
}


