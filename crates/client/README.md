封装 Jupiter Aggregator API，提供链下报价、Swap 构造、Token 列表获取等功能。

包含模块：

- `quote.rs`：单路径报价
- `quote_chain.rs`：多跳报价链组合
- `swap.rs`：生成 swap 报文
- `token_list.rs`：支持 token 名称解析