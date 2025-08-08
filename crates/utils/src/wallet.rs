use solana_sdk::signature::{read_keypair_file, Keypair, Signer};
use std::path::Path;
use crate::{AppError, AppResult};
use tracing::{info, error};

/// 默认钱包路径（也可改为从环境变量读取）
const DEFAULT_KEYPAIR_PATH: &str = "keypairs/arbitrage.json";

/// 加载钱包（适用于 bot、CLI）
pub fn load_wallet(path: Option<&str>) -> AppResult<Keypair> {
    let final_path = path.unwrap_or(DEFAULT_KEYPAIR_PATH);

    info!("🔐 正在加载钱包: {}", final_path);

    let keypair = read_keypair_file(Path::new(final_path))
        .map_err(|e| {
            error!("❌ 钱包加载失败: {}", e);
            AppError::Custom(format!("钱包加载失败: {}", e))
        })?;

    info!("✅ 钱包加载成功，公钥: {}", keypair.pubkey());
    Ok(keypair)
}
