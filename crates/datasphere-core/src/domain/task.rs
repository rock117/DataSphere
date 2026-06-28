use serde::{Deserialize, Serialize};

/// 任务类型。新增数据采集类型时在此扩展，并实现对应执行器。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum TaskType {
    FetchStockList,
    FetchKline,
}

impl TaskType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskType::FetchStockList => "FetchStockList",
            TaskType::FetchKline => "FetchKline",
        }
    }
}

impl std::str::FromStr for TaskType {
    type Err = crate::error::CoreError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "FetchStockList" => Ok(TaskType::FetchStockList),
            "FetchKline" => Ok(TaskType::FetchKline),
            other => Err(crate::error::CoreError::InvalidTaskType(other.to_string())),
        }
    }
}

impl std::fmt::Display for TaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// 触发方式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum TriggerType {
    Cron,
    Manual,
}

impl TriggerType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TriggerType::Cron => "Cron",
            TriggerType::Manual => "Manual",
        }
    }
}

impl std::str::FromStr for TriggerType {
    type Err = crate::error::CoreError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "Cron" => Ok(TriggerType::Cron),
            "Manual" => Ok(TriggerType::Manual),
            other => Err(crate::error::CoreError::InvalidParams(format!(
                "unknown trigger type: {other}"
            ))),
        }
    }
}

impl std::fmt::Display for TriggerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// 运行状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum RunStatus {
    Pending,
    Running,
    Success,
    Failed,
}

impl RunStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            RunStatus::Pending => "Pending",
            RunStatus::Running => "Running",
            RunStatus::Success => "Success",
            RunStatus::Failed => "Failed",
        }
    }
}

impl std::str::FromStr for RunStatus {
    type Err = crate::error::CoreError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "Pending" => Ok(RunStatus::Pending),
            "Running" => Ok(RunStatus::Running),
            "Success" => Ok(RunStatus::Success),
            "Failed" => Ok(RunStatus::Failed),
            other => Err(crate::error::CoreError::InvalidParams(format!(
                "unknown run status: {other}"
            ))),
        }
    }
}

impl std::fmt::Display for RunStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// FetchStockList 任务的参数结构
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FetchStockListParams {
    /// 可选：仅拉取指定市场，None 表示全市场
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub market: Option<super::Market>,
}

/// FetchKline 任务的参数结构
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FetchKlineParams {
    /// 开始日期 (YYYY-MM-DD)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<chrono::NaiveDate>,
    /// 结束日期 (YYYY-MM-DD)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<chrono::NaiveDate>,
    /// 指定股票代码列表，空表示全市场（取 stocks 表）
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub codes: Vec<String>,
}
