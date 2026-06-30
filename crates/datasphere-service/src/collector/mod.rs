//! 采集器抽象层。
//!
//! 每个数据类型对应一个 `Collector` 实现，负责：
//! 1. 解析 params
//! 2. 调 DataSource 拉数据
//! 3. 调 service 层落库
//! 4. 累计 RunStats（含进度更新、取消检查）
//!
//! runner 通过 `CollectorRegistry` 按 `DataType` 查找 Collector，
//! 不关心具体采集逻辑，实现数据类型与调度核心解耦。

pub mod concept;
pub mod fund_holding;
pub mod fund_list;
pub mod industry;
pub mod kline;
pub mod stock_list;

pub use concept::ConceptCollector;
pub use fund_holding::FundHoldingCollector;
pub use fund_list::FundListCollector;
pub use industry::IndustryCollector;
pub use kline::KlineCollector;
pub use stock_list::StockListCollector;

use datasphere_core::domain::{DataType, RunStats};
use datasphere_core::DataSource;
use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use std::sync::Arc;

/// 采集器统一抽象。
///
/// 每个数据类型实现此 trait，注册到 `CollectorRegistry`。
/// runner 通过 `data_type()` 查找对应 Collector，调用 `collect()` 执行。
#[async_trait::async_trait]
pub trait Collector: Send + Sync + 'static {
    /// 处理的数据类型
    fn data_type(&self) -> DataType;

    /// 执行采集。
    ///
    /// runner 已创建 run 记录、已加锁，Collector 只负责：
    /// - 解析 params
    /// - 调 DataSource 拉数据
    /// - 调 service 落库
    /// - 累计 RunStats（含进度更新、取消检查）
    async fn collect(&self, ctx: &CollectContext) -> anyhow::Result<RunStats>;
}

/// 采集上下文，runner 传递给 Collector。
pub struct CollectContext {
    pub db: DatabaseConnection,
    pub source: Arc<dyn DataSource>,
    /// 任务的 params JSON（Collector 按需解析）
    pub params: Option<serde_json::Value>,
    pub run_id: i64,
    /// 更新进度回调：(total, processed)
    pub update_progress: Arc<dyn Fn(usize, usize) + Send + Sync>,
    /// 检查是否取消回调
    pub is_cancelled: Arc<dyn Fn() -> bool + Send + Sync>,
}

/// 采集器注册表。按 `DataType` 注册与查找。
pub struct CollectorRegistry {
    collectors: HashMap<DataType, Arc<dyn Collector>>,
}

impl CollectorRegistry {
    pub fn new() -> Self {
        Self {
            collectors: HashMap::new(),
        }
    }

    /// 注册采集器
    pub fn register(&mut self, collector: Arc<dyn Collector>) {
        self.collectors.insert(collector.data_type(), collector);
    }

    /// 按数据类型查找采集器
    pub fn get(&self, dt: &DataType) -> Option<Arc<dyn Collector>> {
        self.collectors.get(dt).cloned()
    }

    /// 列出所有已注册的数据类型
    pub fn list(&self) -> Vec<DataType> {
        self.collectors.keys().copied().collect()
    }
}

impl Default for CollectorRegistry {
    fn default() -> Self {
        Self::new()
    }
}
