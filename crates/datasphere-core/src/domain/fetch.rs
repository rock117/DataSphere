use crate::domain::{
    Concept, DataType, FundHolding, FundQuote, KlineQuote, StockConcept, StockIndustry, StockQuote,
};
use serde::{Deserialize, Serialize};

/// 统一的拉取参数。
///
/// `data_type` 标识要拉取什么数据，`params` 是各数据类型的具体参数（JSON），
/// 由对应的 Collector 按需解析成强类型结构（如 FetchStockListParams）。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FetchParams {
    pub data_type: DataType,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub params: serde_json::Value,
}

/// 统一的拉取结果。
///
/// 各数据源从 `fetch()` 返回此枚举，Collector 按数据类型 match 取出具体数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FetchResult {
    Stocks(Vec<StockQuote>),
    Industries(Vec<StockIndustry>),
    Concepts(Vec<Concept>, Vec<StockConcept>),
    Funds(Vec<FundQuote>),
    FundHoldings(Vec<FundHolding>),
    Klines(Vec<KlineQuote>),
}
