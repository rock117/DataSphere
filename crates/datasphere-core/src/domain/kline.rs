use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// 日K行情条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlineQuote {
    pub code: String,
    pub date: NaiveDate,
    pub open: f64,
    pub close: f64,
    pub high: f64,
    pub low: f64,
    pub volume: i64,
    pub amount: f64,
    pub turnover: Option<f64>,
    pub pct_change: Option<f64>,
}

/// 数据源拉取日K的请求参数。
/// `code` 为单只股票代码，由 service 层根据 task params 拆分后逐个调用。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchKlineRequest {
    pub code: String,
    pub start: NaiveDate,
    pub end: NaiveDate,
}
