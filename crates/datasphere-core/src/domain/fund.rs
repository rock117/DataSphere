use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// 基金类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum FundType {
    /// 股票型
    Stock,
    /// 混合型
    Mixed,
    /// 债券型
    Bond,
    /// 货币型
    Monetary,
    /// 指数型
    Index,
    /// QDII
    Qdii,
    /// FOF
    Fof,
    /// 其他
    Other,
}

impl FundType {
    pub fn as_str(&self) -> &'static str {
        match self {
            FundType::Stock => "股票型",
            FundType::Mixed => "混合型",
            FundType::Bond => "债券型",
            FundType::Monetary => "货币型",
            FundType::Index => "指数型",
            FundType::Qdii => "QDII",
            FundType::Fof => "FOF",
            FundType::Other => "其他",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "股票型" => FundType::Stock,
            "混合型" => FundType::Mixed,
            "债券型" => FundType::Bond,
            "货币型" => FundType::Monetary,
            "指数型" => FundType::Index,
            "QDII" => FundType::Qdii,
            "FOF" => FundType::Fof,
            _ => FundType::Other,
        }
    }
}

impl std::fmt::Display for FundType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// 基金列表条目（数据源 -> 落库 之间的 DTO）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundQuote {
    pub code: String,
    pub name: String,
    pub fund_type: FundType,
    pub management: String,
    pub custodian: String,
    pub setup_date: Option<NaiveDate>,
    pub latest_scale: Option<f64>,
}
