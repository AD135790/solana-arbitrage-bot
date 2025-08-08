use teloxide::prelude::*;
use utils::resolve_mint_address;
use client::jupiter::quote::fetch_jupiter_quote;
use tracing::{info, error};

/// 处理 /quote 命令，参数格式：/quote SYMBOL1 SYMBOL2 AMOUNT
pub async fn handle_quote(bot: Bot, msg: Message, args: Vec<String>) {
    let chat_id = msg.chat.id;
    let text = msg.text().unwrap_or_default();
    info!("📩 收到消息: {}", text);

    // ✅ 1. 参数校验
    if args.len() != 3 {
        let help_msg = "❗ 用法: /quote SYMBOL1 SYMBOL2 AMOUNT\n例如: /quote SOL USDC 1000000";
        let _ = bot.send_message(chat_id, help_msg).await;
        return;
    }

    // ✅ 2. 提取参数
    let input_symbol = args[0].to_uppercase();
    let output_symbol = args[1].to_uppercase();
    let amount = match args[2].parse::<u64>() {
        Ok(val) => val,
        Err(_) => {
            let _ = bot.send_message(chat_id, "❗ 金额格式错误，应为整数（如 1000000）").await;
            return;
        }
    };

    // ✅ 3. 解析 Mint 地址
    let Some(input_mint) = resolve_mint_address(&input_symbol) else {
        let _ = bot.send_message(chat_id, format!("❌ 不支持的代币符号: {}", input_symbol)).await;
        return;
    };
    let Some(output_mint) = resolve_mint_address(&output_symbol) else {
        let _ = bot.send_message(chat_id, format!("❌ 不支持的代币符号: {}", output_symbol)).await;
        return;
    };

    // ✅ 4. 请求 Jupiter quote
    match fetch_jupiter_quote(input_mint, output_mint, amount).await {
        Ok(quote) => {
            let reply = format!(
                "📊 套利报价成功:\n🔁 {} -> {}\n🔹 输出数量: {}\n🔹 路由平台: {}",
                input_symbol, output_symbol, quote.out_amount, quote.label
            );
            let _ = bot.send_message(chat_id, reply).await;
        }
        Err(e) => {
            error!("❌ 获取 Jupiter 报价失败: {:?}", e);
            let _ = bot.send_message(chat_id, format!("❌ 获取报价失败: {}", e)).await;
        }
    }
}
