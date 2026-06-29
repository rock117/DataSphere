//! 领域模型与 DTO。与 SeaORM entity 解耦，便于数据源层使用纯数据结构。

pub mod concept;
pub mod fund;
pub mod fund_holding;
pub mod kline;
pub mod stock;
pub mod task;

pub use concept::{Concept, StockConcept, StockIndustry};
pub use fund::{FundQuote, FundType};
pub use fund_holding::FundHolding;
pub use kline::{FetchKlineRequest, KlineQuote};
pub use stock::{Market, StockQuote};
pub use task::{
    FetchConceptParams, FetchFundHoldingParams, FetchFundListParams, FetchIndustryParams,
    FetchKlineParams, FetchStockListParams, RunStats, RunStatus, TaskType, TriggerType,
};
