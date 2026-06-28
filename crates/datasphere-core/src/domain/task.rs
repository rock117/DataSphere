use serde::{Deserialize, Serialize};

/// 任务类型。新增数据采集类型时在此扩展，并实现对应执行器。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum TaskType {
    FetchStockList,
    FetchFundList,
    FetchKline,
}

impl TaskType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskType::FetchStockList => "FetchStockList",
            TaskType::FetchFundList => "FetchFundList",
            TaskType::FetchKline => "FetchKline",
        }
    }
}

impl std::str::FromStr for TaskType {
    type Err = crate::error::CoreError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "FetchStockList" => Ok(TaskType::FetchStockList),
            "FetchFundList" => Ok(TaskType::FetchFundList),
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
    /// 部分成功：有成功记录也有失败记录
    Partial,
    /// 已取消：用户请求取消，runner 检测到后提前结束
    Cancelled,
    Failed,
}

impl RunStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            RunStatus::Pending => "Pending",
            RunStatus::Running => "Running",
            RunStatus::Success => "Success",
            RunStatus::Partial => "Partial",
            RunStatus::Cancelled => "Cancelled",
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
            "Partial" => Ok(RunStatus::Partial),
            "Cancelled" => Ok(RunStatus::Cancelled),
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

/// FetchFundList 任务的参数结构
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FetchFundListParams {
    /// 可选：仅拉取指定基金类型，None 表示全类型
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fund_type: Option<super::FundType>,
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

/// 单次任务运行的统计信息。
///
/// runner 在执行过程中累计 success/failed，结束时根据两者关系决定 RunStatus：
/// - failed == 0        -> Success
/// - success > 0 && failed > 0 -> Partial
/// - success == 0 && failed > 0 -> Failed
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunStats {
    pub success: usize,
    pub failed: usize,
    /// 失败明细（每条失败一个简短描述，用于 error 字段汇总）
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,
    /// 运行耗时（毫秒），由 runner 在结束时计算
    pub duration_ms: u64,
}

impl RunStats {
    pub fn record_success(&mut self, n: usize) {
        self.success += n;
    }

    pub fn record_failure(&mut self, msg: impl Into<String>) {
        self.failed += 1;
        self.errors.push(msg.into());
    }

    /// 根据成功/失败数推断最终状态
    pub fn derive_status(&self) -> RunStatus {
        if self.failed == 0 {
            RunStatus::Success
        } else if self.success > 0 {
            RunStatus::Partial
        } else {
            RunStatus::Failed
        }
    }

    /// 拼接错误明细为单条字符串（用于 task_runs.error 字段）
    pub fn error_summary(&self) -> Option<String> {
        if self.errors.is_empty() {
            None
        } else {
            Some(format!(
                "{} failure(s): {}",
                self.failed,
                self.errors.join("; ")
            ))
        }
    }

    pub fn total(&self) -> usize {
        self.success + self.failed
    }
}
