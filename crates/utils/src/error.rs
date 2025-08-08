use thiserror::Error;

/// 项目通用错误类型
#[derive(Debug, Error)]
pub enum AppError {
    #[error("自定义错误: {0}")]
    Custom(String),

    #[error("网络请求失败: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("JSON 解析失败: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("未找到报价路径")]
    NoQuoteFound,

    #[error("未知错误: {0}")]
    Other(String),

    #[error("JSON 解析失败: {0}")]
    ParseError(String),

    #[error("外部服务错误: {0}")]
    External(String),
}

/// 项目统一返回类型
pub type AppResult<T> = Result<T, AppError>;
