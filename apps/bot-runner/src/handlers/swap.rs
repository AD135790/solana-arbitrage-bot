use teloxide::prelude::*;
use client::jupiter::swap::fetch_jupiter_swap_tx_safe;
use solana_sdk::{signature::Signer, transaction::Transaction};
use solana_client::nonblocking::rpc_client::RpcClient;
use base64::prelude::*;
use tracing::{info, error, warn};
use utils::wallet::load_wallet;
use std::time::Instant;
use utils::resolve_mint_address;

/// 处理 /swap 命令的核心逻辑
pub async fn handle_swap(
    bot: Bot,
    msg: Message,
    input_token: String,
    output_token: String,
    amount: f64,
) {
    let chat_id = msg.chat.id;

    info!("📩 /swap 请求: {} -> {}, 数量: {}", input_token, output_token, amount);

    if amount <= 0.0 {
        let _ = bot.send_message(chat_id, "❗ 金额必须大于 0").await;
        return;
    }

    let _ = bot.send_message(chat_id, format!("🔄 正在兑换 {} → {}，数量 {}", input_token, output_token, amount)).await;

    // 1. Mint 地址解析
    let Some(input_mint) = resolve_mint_address(&input_token) else {
        error!("❌ 未识别输入代币: {}", input_token);
        let _ = bot.send_message(chat_id, format!("❌ 不支持的输入币种: {}", input_token)).await;
        return;
    };
    let Some(output_mint) = resolve_mint_address(&output_token) else {
        error!("❌ 未识别输出代币: {}", output_token);
        let _ = bot.send_message(chat_id, format!("❌ 不支持的输出币种: {}", output_token)).await;
        return;
    };

    // 2. 加载钱包
    let keypair = match load_wallet(None) {
        Ok(k) => k,
        Err(err) => {
            error!("🔐 钱包加载失败: {}", err);
            let _ = bot.send_message(chat_id, format!("🔐 钱包加载失败: {}", err)).await;
            return;
        }
    };
    let user_pubkey = keypair.pubkey();
    info!("✅ 钱包加载成功: {}", user_pubkey);

    // 3. 请求 Jupiter swap 交易体
    let t1 = Instant::now();
    let swap_tx_base64 = match fetch_jupiter_swap_tx_safe(
        &input_mint,
        &output_mint,
        amount,
        9, // TODO: 可根据 token 设置 decimals
        &user_pubkey.to_string(),
    ).await {
        Ok(tx) => {
            info!("📦 Jupiter 返回交易体 base64 长度: {}, 用时: {:?}", tx.len(), t1.elapsed());
            tx
        }
        Err(err) => {
            warn!("🧪 Jupiter swap 请求失败，使用 mock 替代。错误: {}", err);
            let _ = bot.send_message(chat_id, "⚠️ Jupiter 出错，使用测试交易体替代执行").await;

            // ✅ 你需替换为有效的 base64（手动抓日志或控制台打印）
            let mock_base64 = "AgABBAIBAgMEBQYHCAkKCwwNDg8QERITFBUWFxgZGhs=";
            mock_base64.to_string()
        }
    };

    // 4. 解码 base64
    let tx_data = match BASE64_STANDARD.decode(&swap_tx_base64) {
        Ok(data) => data,
        Err(err) => {
            error!("❌ base64 解码失败: {:?}", err);
            let _ = bot.send_message(chat_id, "❌ Jupiter 返回数据格式有误（base64 解析失败）").await;
            return;
        }
    };

    // 5. bincode 反序列化 Transaction
    let mut tx: Transaction = match bincode::deserialize::<Transaction>(&tx_data) {
        Ok(tx) => {
            info!("🧾 Transaction 解析成功，签名数: {}", tx.signatures.len());
            tx
        }
        Err(err) => {
            error!("❌ bincode 反序列化失败: {:?}", err);
            let _ = bot.send_message(chat_id, "❌ 交易结构解析失败").await;
            return;
        }
    };

    // 6. 获取最新 Blockhash
    let rpc = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());
    let bh_start = Instant::now();
    let recent_blockhash = match rpc.get_latest_blockhash().await {
        Ok(bh) => {
            info!("⛓️ 获取最新 Blockhash 成功，用时: {:?}", bh_start.elapsed());
            bh
        }
        Err(err) => {
            error!("❌ 获取 Blockhash 失败: {:?}", err);
            let _ = bot.send_message(chat_id, "❌ 获取最新 Blockhash 失败").await;
            return;
        }
    };
    tx.message.recent_blockhash = recent_blockhash;

    // 7. 签名
    tx.sign(&[&keypair], recent_blockhash);
    info!("✍️ 已签名，开始广播");

    // 8. 广播交易
    match rpc.send_and_confirm_transaction(&tx).await {
        Ok(sig) => {
            let url = format!("https://solscan.io/tx/{}", sig);
            info!("✅ 交易成功，哈希: {}", sig);
            let _ = bot.send_message(chat_id, format!("✅ 成功！交易哈希:\n{}", url)).await;
        }
        Err(err) => {
            error!("❌ 交易广播失败: {:?}", err);
            let _ = bot.send_message(chat_id, format!("❌ 交易失败: {}", err)).await;
        }
    }
}
