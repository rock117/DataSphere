use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// 交易所/市场标识
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Market {
    SH,
    SZ,
    BJ,
}

impl Market {
    pub fn as_str(&self) -> &'static str {
        match self {
            Market::SH => "SH",
            Market::SZ => "SZ",
            Market::BJ => "BJ",
        }
    }

    pub fn exchange(&self) -> &'static str {
        match self {
            Market::SH => "上海证券交易所",
            Market::SZ => "深圳证券交易所",
            Market::BJ => "北京证券交易所",
        }
    }

    /// 根据股票代码推断市场（A股规则：6开头沪市，0/3开头深市，8/4开头北交所）
    pub fn from_code(code: &str) -> Self {
        match code.chars().next() {
            Some('6') => Market::SH,
            Some('0') | Some('3') => Market::SZ,
            Some('8') | Some('4') => Market::BJ,
            _ => Market::SZ,
        }
    }
}

impl std::fmt::Display for Market {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Market {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_ascii_uppercase().as_str() {
            "SH" => Ok(Market::SH),
            "SZ" => Ok(Market::SZ),
            "BJ" => Ok(Market::BJ),
            other => Err(format!("unknown market: {other}")),
        }
    }
}

/// 股票列表条目（数据源 -> 落库 之间的 DTO）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockQuote {
    pub code: String,
    pub symbol: String,
    pub name: String,
    pub market: Market,
    pub exchange: String,
    pub list_date: Option<NaiveDate>,
    pub delist_date: Option<NaiveDate>,
}

impl StockQuote {
    pub fn new(code: impl Into<String>, name: impl Into<String>, market: Market) -> Self {
        let code = code.into();
        let symbol = format!("{}{}", market.as_str().to_lowercase(), code);
        Self {
            code,
            symbol,
            name: name.into(),
            market,
            exchange: market.exchange().to_string(),
            list_date: None,
            delist_date: None,
        }
    }
}
