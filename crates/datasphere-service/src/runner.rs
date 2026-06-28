use datasphere_core::domain::{
    FetchKlineParams, FetchStockListParams, RunStats, RunStatus, TaskType, TriggerType,
};
use datasphere_core::DataSourceRegistry;
use datasphere_entity::task;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use std::time::Instant;

use crate::kline_service::KlineService;
use crate::stock_service::StockService;
use crate::task_service::TaskService;

/// 任务执行器。
///
/// 持有 `DataSourceRegistry` 和 DB 连接，根据 task_type 路由到对应逻辑：
/// 1. 从 Registry 获取 provider 对应的 DataSource
/// 2. 调用 DataSource 拉取数据
/// 3. 调用 service 层 upsert 落库
/// 4. 记录 task_run（含成功/失败统计、耗时、进度）
///
/// 执行策略：
/// - 单只股票失败不中断整个任务，继续处理其他，最后汇总为 RunStats
/// - 循环中定期检查 cancel 标志，被取消时提前结束并标记 Cancelled
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
    /// - 按 task_type 路由执行，累计 RunStats，更新进度，检查取消
    /// - 根据 RunStats 推断最终状态（Success / Partial / Failed / Cancelled）并更新 task_run
    /// 返回 (run_id, 最终状态)
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
                RunStatus::Failed,
                &RunStats::default(),
                Some(msg.clone()),
            )
            .await?;
            anyhow::bail!(msg);
        }
        self.locks.insert(task_model.id, ());

        let started = Instant::now();
        let result = self.execute_inner(task_model, run_id).await;
        let duration_ms = started.elapsed().as_millis() as u64;

        // 释放锁
        self.locks.remove(&task_model.id);

        match result {
            Ok(mut stats) => {
                stats.duration_ms = duration_ms;
                // 检查是否被取消
                let cancelled = TaskService::is_cancel_requested(&self.db, run_id)
                    .await
                    .unwrap_or(false);
                let status = if cancelled && stats.success == 0 {
                    RunStatus::Cancelled
                } else if cancelled {
                    RunStatus::Partial // 已有部分成功，取消后记为 Partial
                } else {
                    stats.derive_status()
                };
                let error = if cancelled {
                    Some(format!(
                        "cancelled by user after {} processed",
                        stats.total()
                    ))
                } else {
                    stats.error_summary()
                };
                tracing::info!(
                    "task {} run {} finished: status={} success={} failed={} duration_ms={}",
                    task_model.id,
                    run_id,
                    status,
                    stats.success,
                    stats.failed,
                    stats.duration_ms
                );
                TaskService::finish_run(&self.db, run_id, status, &stats, error.clone()).await?;
                if status == RunStatus::Failed {
                    anyhow::bail!(
                        "task {} run {} failed: {}",
                        task_model.id,
                        run_id,
                        error.unwrap_or_default()
                    );
                }
                Ok(run_id)
            }
            Err(e) => {
                let msg = format!("{e:#}");
                tracing::error!("task {} run {} errored: {msg}", task_model.id, run_id);
                let stats = RunStats {
                    duration_ms,
                    ..Default::default()
                };
                TaskService::finish_run(&self.db, run_id, RunStatus::Failed, &stats, Some(msg))
                    .await?;
                Err(e)
            }
        }
    }

    /// 内部执行逻辑：解析 task_type 与 params，调用对应数据源。
    ///
    /// 单个数据单元（单只股票 / 单次拉取）失败不中断，计入 RunStats 后继续。
    /// 每轮循环检查 cancel 标志。
    async fn execute_inner(
        &self,
        task_model: &task::Model,
        run_id: i64,
    ) -> anyhow::Result<RunStats> {
        let task_type: TaskType = task_model.task_type.parse()?;
        let source = self.registry.get(&task_model.provider)?;
        let mut stats = RunStats::default();

        match task_type {
            TaskType::FetchStockList => {
                let params: FetchStockListParams = task_model
                    .params
                    .as_ref()
                    .map(|v| serde_json::from_value(v.clone()))
                    .transpose()?
                    .unwrap_or_default();

                // 单次拉取，total=1
                TaskService::update_progress(&self.db, run_id, 1, 0)
                    .await
                    .ok();

                tracing::info!("Fetching stock list via provider={}", task_model.provider);
                if Self::is_cancelled(&self.db, run_id).await {
                    return Ok(stats);
                }
                match source.fetch_stock_list(&params).await {
                    Ok(quotes) => match StockService::upsert_many(&self.db, &quotes).await {
                        Ok(n) => stats.record_success(n),
                        Err(e) => stats.record_failure(format!("upsert stock list: {e:#}")),
                    },
                    Err(e) => stats.record_failure(format!("fetch stock list: {e:#}")),
                }
                TaskService::update_progress(&self.db, run_id, 1, 1)
                    .await
                    .ok();
                tracing::info!(
                    "Stock list done: success={} failed={}",
                    stats.success,
                    stats.failed
                );
            }
            TaskType::FetchKline => {
                let params: FetchKlineParams = task_model
                    .params
                    .as_ref()
                    .map(|v| serde_json::from_value(v.clone()))
                    .transpose()?
                    .unwrap_or_default();

                // codes 为空时取全市场股票列表
                let default_codes = match StockService::list_all_codes(&self.db).await {
                    Ok(c) => c,
                    Err(e) => {
                        stats.record_failure(format!("load stock codes: {e:#}"));
                        return Ok(stats);
                    }
                };
                let requests = params.to_requests(&default_codes);
                let total = requests.len();
                TaskService::update_progress(&self.db, run_id, total, 0)
                    .await
                    .ok();

                for (i, req) in requests.iter().enumerate() {
                    // 检查取消
                    if Self::is_cancelled(&self.db, run_id).await {
                        tracing::info!(
                            "task {} run {} cancelled at {}/{}",
                            task_model.id,
                            run_id,
                            i,
                            total
                        );
                        break;
                    }
                    tracing::info!(
                        "Fetching kline [{}/{}] code={} start={} end={}",
                        i + 1,
                        total,
                        req.code,
                        req.start,
                        req.end
                    );
                    match source.fetch_kline(req).await {
                        Ok(quotes) => match KlineService::upsert_many(&self.db, &quotes).await {
                            Ok(n) => stats.record_success(n),
                            Err(e) => stats
                                .record_failure(format!("upsert kline code={}: {e:#}", req.code)),
                        },
                        Err(e) => {
                            stats.record_failure(format!("fetch kline code={}: {e:#}", req.code))
                        }
                    }
                    TaskService::update_progress(&self.db, run_id, total, i + 1)
                        .await
                        .ok();
                }
                tracing::info!(
                    "Kline batch done: success={} failed={}",
                    stats.success,
                    stats.failed
                );
            }
        }

        Ok(stats)
    }

    /// 检查某次运行是否被请求取消
    async fn is_cancelled(db: &DatabaseConnection, run_id: i64) -> bool {
        TaskService::is_cancel_requested(db, run_id)
            .await
            .unwrap_or(false)
    }
}
