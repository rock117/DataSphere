//! 数据源抽象层。
//!
//! `DataSource` trait 定义了所有数据源的统一接口。
//! `DataSourceRegistry` 按 provider 名注册与查找，实现"可切换"。
//! 内置 `MockDataSource` 生成假数据。

pub mod mock;
pub mod registry;

pub use mock::MockDataSource;
pub use registry::DataSourceRegistry;

use crate::domain::{FetchKlineRequest, KlineQuote, StockQuote};
use crate::error::Result;
use async_trait::async_trait;

/// 数据源统一抽象。
///
/// 实现此 trait 即可接入新的数据源（如 Tushare、AKShare、Yahoo Finance 等），
/// 然后在 `DataSourceRegistry` 中注册，任务通过 `provider` 字段选择数据源。
#[async_trait]
pub trait DataSource: Send + Sync + 'static {
    /// 数据源名称，用于在 Registry 中查找（如 "mock"、"tushare"）
    fn name(&self) -> &str;

    /// 拉取 A股股票列表
    async fn fetch_stock_list(
        &self,
        params: &crate::domain::FetchStockListParams,
    ) -> Result<Vec<StockQuote>>;

    /// 拉取单只股票的日K历史行情
    async fn fetch_kline(&self, req: &FetchKlineRequest) -> Result<Vec<KlineQuote>>;
}
