use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// 基金成分股（持仓明细）条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundHolding {
    pub fund_code: String,
    pub stock_code: String,
    pub stock_name: String,
    /// 报告期（季报披露日）
    pub report_date: NaiveDate,
    /// 占净值比例(%)
    pub weight: f64,
    /// 持仓股数
    pub shares: Option<i64>,
    /// 持仓市值(元)
    pub market_value: Option<f64>,
    /// 持仓排名（第几大）
    pub rank: Option<i32>,
}
