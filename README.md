# 🧠 Solana Arbitrage Bot

> 构建一个 Rust + Anchor 驱动的链上套利机器人，支持 CLI 工具 + Telegram Bot 控制，后续接入闪电贷、路径模拟、奖励机制等功能。

---

## 🚀 项目目标

这个项目旨在成为一个模块化、可扩展的 Solana 链上套利系统，支持：

- 🔍 获取最佳报价路径（Jupiter Aggregator）
- 💬 Telegram 命令触发套利请求
- 🧪 CLI 本地命令测试套利路径
- 💸 支持后续闪电贷与策略部署

---

## 🧱 项目结构

```bash
apps/                 # 应用层
├── bot-runner        # Telegram Bot
├── cli-runner        # 命令行工具

crates/               # 核心逻辑
├── client            # Jupiter 报价与 swap 封装
├── arbitrage         # 套利路径评估（规划中）
├── executor          # 交易发起器（规划中）
└── utils             # 通用工具库

docs/
└── day1-log.md       # 每日开发记录

keypairs/             # 本地测试钱包（不上传 GitHub）
