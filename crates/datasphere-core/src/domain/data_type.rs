use serde::{Deserialize, Serialize};

/// 数据类型标识。
///
/// 编译期穷尽性检查：加新数据类型在此扩展 enum，
/// 编译器会在所有 match 处提示需要补充。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataType {
    #[default]
    StockList,
    Industry,
    Concept,
    FundList,
    FundHolding,
    Kline,
}

impl DataType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DataType::StockList => "stock_list",
            DataType::Industry => "industry",
            DataType::Concept => "concept",
            DataType::FundList => "fund_list",
            DataType::FundHolding => "fund_holding",
            DataType::Kline => "kline",
        }
    }
}

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for DataType {
    type Err = crate::error::CoreError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "stock_list" => Ok(DataType::StockList),
            "industry" => Ok(DataType::Industry),
            "concept" => Ok(DataType::Concept),
            "fund_list" => Ok(DataType::FundList),
            "fund_holding" => Ok(DataType::FundHolding),
            "kline" => Ok(DataType::Kline),
            other => Err(crate::error::CoreError::InvalidTaskType(other.to_string())),
        }
    }
}
