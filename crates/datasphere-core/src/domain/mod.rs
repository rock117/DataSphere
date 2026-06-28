//! 领域模型与 DTO。与 SeaORM entity 解耦，便于数据源层使用纯数据结构。

pub mod kline;
pub mod stock;
pub mod task;

pub use kline::{FetchKlineRequest, KlineQuote};
pub use stock::{Market, StockQuote};
pub use task::{
    FetchKlineParams, FetchStockListParams, RunStats, RunStatus, TaskType, TriggerType,
};
