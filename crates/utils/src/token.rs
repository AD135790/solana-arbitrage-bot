use std::collections::HashMap;
use once_cell::sync::Lazy;

/// 静态 Token 映射表：symbol → mint address
pub static TOKEN_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    HashMap::from([
        // 🔹 主流代币
        ("SOL",      "So11111111111111111111111111111111111111112"),
        ("USDC",     "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
        ("USDT",     "Es9vMFrzaCERD4JcWta4dWh8uWAWN8ZVnZ4DJpK33VCU"),
        ("BONK",     "DezX1FS7Lm7nFf6TgCKzZB5uEJuYH1a4FJkT3wqVRSg"),

        // 🔹 Staking 衍生品
        ("MSOL",     "mSoLz5v4v2pJRoyzZ4XzUp5wUJGFEBq4kF4kCNhMtwk"),
        ("JITOSOL",  "jitoS1Ztqj3AzFTsni8VvfdYDQNKRYskBA6EUX7FU6h"),
        ("BSOL",     "ABKBACBNfj5tChNpm1MNPCoxHktsKWFzxgs5LmuCohS3"),

        // 可扩展更多...
    ])
});

/// 获取 symbol 对应的 Mint 地址（如 "SOL" -> "So111..."）
pub fn resolve_mint_address(symbol: &str) -> Option<&'static str> {
    TOKEN_MAP.get(&symbol.to_uppercase().as_str()).copied()
}
