use teloxide::{prelude::*, respond};
use dotenvy::dotenv;
use tracing::{error, info};
mod commands;
mod handlers;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let bot = Bot::from_env();
    println!("🤖 启动套利机器人...");

    // ✅ 显式检查 bot 连接是否成功（避免 panic）
    match bot.get_me().send().await {
        Ok(me) => {
            info!("✅ Bot 启动成功，用户名: @{}", me.user.username.unwrap_or_default());
        }
        Err(err) => {
            error!("❌ 启动失败，无法连接 Telegram API: {}", err);
            eprintln!("❌ 启动失败，请检查网络连接或代理设置（如 HTTPS_PROXY）");
            return;
        }
    }

    // ✅ 进入命令循环
    teloxide::repl(bot.clone(), move |message: Message| {
        let bot = bot.clone();
        async move {
            if let Some(text) = message.text() {
                commands::handle_command(bot, message.clone(), text).await;
            }
            respond(())
        }
    })
    .await;
}