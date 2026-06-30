//! 数据源抽象层。
//!
//! `DataSource` trait 定义了所有数据源的统一接口。
//! 数据源通过 `capabilities()` 声明支持哪些 `DataType`，
//! 通过统一的 `fetch()` 方法按 `DataType` 分发到具体拉取逻辑。
//!
//! 新增数据类型时，数据源只需在 `capabilities()` 加声明 + `fetch()` 加分发，
//! 不需要改 trait 定义本身。
//!
//! `DataSourceRegistry` 按 provider 名注册与查找，实现"可切换"。

pub mod mock;
pub mod registry;

pub use mock::MockDataSource;
pub use registry::DataSourceRegistry;

use crate::domain::{DataType, FetchParams, FetchResult};
use crate::error::Result;
use async_trait::async_trait;

/// 数据源统一抽象。
///
/// 数据源与数据类型解耦：
/// - 数据源声明自己支持哪些 `DataType`（capabilities）
/// - 统一的 `fetch()` 入口按 `DataType` 分发
/// - runner 在执行前校验数据源是否支持目标数据类型
#[async_trait]
pub trait DataSource: Send + Sync + 'static {
    /// 数据源名称，用于在 Registry 中查找（如 "mock"、"tushare"）
    fn name(&self) -> &str;

    /// 声明支持的数据类型列表
    fn capabilities(&self) -> &[DataType];

    /// 统一拉取入口，按 `data_type` 分发到具体拉取逻辑。
    ///
    /// 数据源实现时内部 match `params.data_type`，调用对应的 API。
    async fn fetch(&self, params: &FetchParams) -> Result<FetchResult>;
}
