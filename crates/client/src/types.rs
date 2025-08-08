/// Jupiter 报价结构体（可扩展）
#[derive(Debug)]
pub struct QuoteInfo {
    pub out_amount: String,
    pub label: String,
    
}


impl QuoteInfo {
    /// 输出中文格式的套利信息，方便调试或接入 Bot
    pub fn display_chinese(&self) {
        println!("📊 套利报价结果：");
        println!("🔹 输出目标代币数量: {}", self.out_amount);
        println!("🔹 路由 AMM 平台: {}", self.label);
    }

    /// 返回 Bot 可用的消息格式
    pub fn to_bot_message(&self) -> String {
        format!(
            "📈 当前套利报价：\n💰 输出数量：{}\n🔀 通过 AMM：{}\n",
            self.out_amount, self.label
        )
    }
}

#[derive(Debug, Clone)]
pub struct QuoteRoute {
    pub out_amount: String,
    pub label: String,
    pub hops: usize,
}
