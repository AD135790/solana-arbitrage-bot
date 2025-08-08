use teloxide::prelude::*;
use crate::handlers::swap::handle_swap;

/// /swap SOL USDC 0.1
pub async fn handle_swap_command(bot: Bot, msg: Message, args: Vec<String>) {
    if args.len() != 3 {
        bot.send_message(msg.chat.id, "❗ 用法: /swap <输入币种> <输出币种> <数量>")
            .await.ok();
        return;
    }

    let input_token = args[0].to_uppercase();
    let output_token = args[1].to_uppercase();
    let amount = match args[2].parse::<f64>() {
        Ok(val) => val,
        Err(_) => {
            bot.send_message(msg.chat.id, "❗ 金额格式错误，应为数字")
                .await.ok();
            return;
        }
    };

    handle_swap(bot, msg, input_token, output_token, amount).await;
}
