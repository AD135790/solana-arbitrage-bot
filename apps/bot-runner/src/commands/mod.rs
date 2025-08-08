pub mod quote;
pub mod swap; // 新增模块

use teloxide::prelude::*;
use quote::handle_quote;
use swap::handle_swap_command;

pub async fn handle_command(bot: Bot, msg: Message, text: &str) {
    let mut parts = text.trim().split_whitespace();
    let command = parts.next().unwrap_or("");
    let args: Vec<String> = parts.map(|s| s.to_string()).collect();

    match command {
        "/quote" => {
            handle_quote(bot, msg, args).await;
        }
        "/swap" => {
            handle_swap_command(bot, msg, args).await;
        }
        _ => {
            bot.send_message(msg.chat.id, "❓ 不支持的命令").await.ok();
        }
    }
}
