use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs, path::PathBuf};

#[derive(Serialize, Deserialize)]
struct MintCache {
    mints: Vec<String>,
}

fn cache_path() -> Result<PathBuf> {
    let dir = dirs::cache_dir().context("找不到系统缓存目录")?.join("solarb"); // 自定义目录
    fs::create_dir_all(&dir).ok();
    Ok(dir.join("jup_tradable_v1.json"))
}

pub fn save_mints(set: &HashSet<String>) -> Result<()> {
    let path = cache_path()?;
    let payload = MintCache { mints: set.iter().cloned().collect() };
    let bytes = serde_json::to_vec_pretty(&payload)?;
    fs::write(&path, bytes)?;
    Ok(())
}

pub fn load_mints() -> Result<HashSet<String>> {
    let path = cache_path()?;
    let bytes = fs::read(&path)?;
    let payload: MintCache = serde_json::from_slice(&bytes)?;
    Ok(payload.mints.into_iter().collect())
}
