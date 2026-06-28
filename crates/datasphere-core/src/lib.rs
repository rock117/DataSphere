//! DataSphere 核心抽象层。
//!
//! 包含领域 DTO、错误类型、`DataSource` 抽象与注册表、内置 Mock 数据源。
//! 不依赖任何数据库或 Web 框架，便于在 service / api / 测试中复用。

pub mod datasource;
pub mod domain;
pub mod error;

pub use datasource::{DataSource, DataSourceRegistry, MockDataSource};
pub use domain::{
    FetchKlineParams, FetchKlineRequest, FetchStockListParams, KlineQuote, Market, RunStats,
    StockQuote,
};
pub use domain::{RunStatus, TaskType, TriggerType};
pub use error::{CoreError, Result};
