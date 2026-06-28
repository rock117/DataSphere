use datasphere_core::domain::{FetchKlineParams, FetchStockListParams, TaskType, TriggerType};
use datasphere_core::DataSourceRegistry;
use datasphere_entity::task;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

use crate::kline_service::KlineService;
use crate::stock_service::StockService;
use crate::task_service::TaskService;

/// 任务执行器。
///
/// 持有 `DataSourceRegistry` 和 DB 连接，根据 task_type 路由到对应逻辑：
/// 1. 从 Registry 获取 provider 对应的 DataSource
/// 2. 调用 DataSource 拉取数据
/// 3. 调用 service 层 upsert 落库
/// 4. 记录 task_run
pub struct TaskRunner {
    db: DatabaseConnection,
    registry: Arc<DataSourceRegistry>,
    /// 任务级互斥锁，防止同一任务并发执行
    locks: dashmap::DashMap<i64, ()>,
}

impl TaskRunner {
    pub fn new(db: DatabaseConnection, registry: Arc<DataSourceRegistry>) -> Self {
        Self {
            db,
            registry,
            locks: dashmap::DashMap::new(),
        }
    }

    /// 执行指定任务。
    ///
    /// - 创建 Running 状态的 task_run
    /// - 获取任务级锁（同一 task 不允许并发）
    /// - 按 task_type 路由执行
    /// - 更新 task_run 为 Success / Failed
    /// 返回 run_id
    pub async fn run(&self, task_model: &task::Model, trigger: TriggerType) -> anyhow::Result<i64> {
        // 创建运行记录
        let run = TaskService::create_run(&self.db, task_model.id, trigger).await?;
        let run_id = run.id;

        // 尝试获取任务级锁
        if self.locks.contains_key(&task_model.id) {
            let msg = format!("task {} is already running", task_model.id);
            TaskService::finish_run(
                &self.db,
                run_id,
                datasphere_core::domain::RunStatus::Failed,
                0,
                Some(msg.clone()),
            )
            .await?;
            anyhow::bail!(msg);
        }
        self.locks.insert(task_model.id, ());

        let result = self.execute_inner(task_model).await;

        // 释放锁
        self.locks.remove(&task_model.id);

        match result {
            Ok(records) => {
                TaskService::finish_run(
                    &self.db,
                    run_id,
                    datasphere_core::domain::RunStatus::Success,
                    records,
                    None,
                )
                .await?;
                Ok(run_id)
            }
            Err(e) => {
                let msg = format!("{e:#}");
                tracing::error!("task {} run {} failed: {msg}", task_model.id, run_id);
                TaskService::finish_run(
                    &self.db,
                    run_id,
                    datasphere_core::domain::RunStatus::Failed,
                    0,
                    Some(msg),
                )
                .await?;
                Err(e)
            }
        }
    }

    /// 内部执行逻辑：解析 task_type 与 params，调用对应数据源
    async fn execute_inner(&self, task_model: &task::Model) -> anyhow::Result<usize> {
        let task_type: TaskType = task_model.task_type.parse()?;
        let source = self.registry.get(&task_model.provider)?;

        match task_type {
            TaskType::FetchStockList => {
                let params: FetchStockListParams = task_model
                    .params
                    .as_ref()
                    .map(|v| serde_json::from_value(v.clone()))
                    .transpose()?
                    .unwrap_or_default();

                tracing::info!("Fetching stock list via provider={}", task_model.provider);
                let quotes = source.fetch_stock_list(&params).await?;
                let count = StockService::upsert_many(&self.db, &quotes).await?;
                tracing::info!("Fetched {} stocks", count);
                Ok(count)
            }
            TaskType::FetchKline => {
                let params: FetchKlineParams = task_model
                    .params
                    .as_ref()
                    .map(|v| serde_json::from_value(v.clone()))
                    .transpose()?
                    .unwrap_or_default();

                // codes 为空时取全市场股票列表
                let default_codes = StockService::list_all_codes(&self.db).await?;
                let requests = params.to_requests(&default_codes);

                let mut total = 0;
                for req in requests {
                    tracing::info!(
                        "Fetching kline code={} start={} end={}",
                        req.code,
                        req.start,
                        req.end
                    );
                    let quotes = source.fetch_kline(&req).await?;
                    let n = KlineService::upsert_many(&self.db, &quotes).await?;
                    total += n;
                }
                tracing::info!("Fetched {} kline records in total", total);
                Ok(total)
            }
        }
    }
}
