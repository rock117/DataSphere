use serde::{Deserialize, Serialize};

/// 股票行业分类（申万一级）
/// 一只股票一个行业，拉取时给每只股票设置 industry 字段
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StockIndustry {
    pub code: String,
    pub industry: String,
}

/// 概念板块条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concept {
    pub name: String,
    pub description: Option<String>,
}

/// 股票-概念关联条目（拉取时用：股票代码 + 概念名）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockConcept {
    pub stock_code: String,
    pub concept_name: String,
}
