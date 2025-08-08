use teloxide::{prelude::*, utils::markdown::escape};
use client::jupiter::quote::fetch_jupiter_quote;
use utils::resolve_mint_address;
use tracing::{info, warn};

pub async fn handle_quote(bot: Bot, msg: Message, args: Vec<String>) {
    let chat_id = msg.chat.id;
    let _ = bot.send_message(chat_id, "🧾 正在处理 /quote 命令...").await;

    // ✅ Step 1: 参数校验
    if args.len() != 3 {
        let _ = bot.send_message(chat_id, "❗ 用法: /quote SYMBOL1 SYMBOL2 AMOUNT\n例如: /quote SOL USDC 1000000").await;
        return;
    }

    let input = args[0].to_uppercase();
    let output = args[1].to_uppercase();
    let amount = match args[2].parse::<u64>() {
        Ok(val) => val,
        Err(_) => {
            let _ = bot.send_message(chat_id, "❗ 金额格式错误，应为整数（如 1000000）").await;
            return;
        }
    };
    info!("📥 quote 请求参数: {} -> {}, 数量: {}", input, output, amount);

    // ✅ Step 2: 解析 Mint 地址
    let Some(input_mint) = resolve_mint_address(&input) else {
        let _ = bot.send_message(chat_id, format!("❌ 不支持的输入代币: {}", input)).await;
        return;
    };
    let Some(output_mint) = resolve_mint_address(&output) else {
        let _ = bot.send_message(chat_id, format!("❌ 不支持的输出代币: {}", output)).await;
        return;
    };

    // ✅ Step 3: 调用 Jupiter Quote 接口
    match fetch_jupiter_quote(input_mint, output_mint, amount).await {
        Ok(quote) => {
            let reply = format!(
                "📊 套利报价结果：\n🔁 {} → {}\n🔹 输出数量: {}\n🔹 路由平台: {}",
                escape(&input),
                escape(&output),
                quote.out_amount,
                quote.label
            );
            let _ = bot.send_message(chat_id, reply).await;
        }

        Err(e) => {
            warn!("🧪 quote 请求失败，使用 mock 替代: {:?}", e);
            let reply = format!(
                "📊 [Mock] quote 成功：\n🔁 {} → {}\n🔹 输出数量: {}\n🔹 路由平台: JupiterMock",
                escape(&input),
                escape(&output),
                999_999
            );
            let _ = bot.send_message(chat_id, reply).await;
        }
    }
}
