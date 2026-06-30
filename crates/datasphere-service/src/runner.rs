use datasphere_core::domain::{DataType, RunStats, RunStatus, TriggerType};
use datasphere_core::DataSourceRegistry;
use datasphere_entity::task;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use std::time::Instant;

use crate::collector::{CollectContext, CollectorRegistry};
use crate::task_service::TaskService;

/// 任务执行器（调度核心）。
///
/// 职责：
/// - 创建/完成 task_run 记录
/// - 任务级互斥锁（同一 task 不允许并发）
/// - 进度更新 + 取消检查（通过 CollectContext 回调传给 Collector）
/// - 按 DataType 查找 Collector 并执行
///
/// **不关心具体采集逻辑**——每种数据类型的采集由对应的 Collector 实现，
/// runner 通过 CollectorRegistry 查找，加新数据类型不需要改 runner。
pub struct TaskRunner {
    db: DatabaseConnection,
    registry: Arc<DataSourceRegistry>,
    collectors: Arc<CollectorRegistry>,
    locks: dashmap::DashMap<i64, ()>,
}

impl TaskRunner {
    pub fn new(
        db: DatabaseConnection,
        registry: Arc<DataSourceRegistry>,
        collectors: Arc<CollectorRegistry>,
    ) -> Self {
        Self {
            db,
            registry,
            collectors,
            locks: dashmap::DashMap::new(),
        }
    }

    /// 执行指定任务。
    pub async fn run(&self, task_model: &task::Model, trigger: TriggerType) -> anyhow::Result<i64> {
        let run = TaskService::create_run(&self.db, task_model.id, trigger).await?;
        let run_id = run.id;

        // 任务级互斥锁
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

        self.locks.remove(&task_model.id);

        match result {
            Ok(mut stats) => {
                stats.duration_ms = duration_ms;
                let cancelled = TaskService::is_cancel_requested(&self.db, run_id)
                    .await
                    .unwrap_or(false);
                let status = if cancelled && stats.success == 0 {
                    RunStatus::Cancelled
                } else if cancelled {
                    RunStatus::Partial
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

    /// 内部执行：查找 Collector + 校验数据源能力 + 构造上下文 + 执行
    async fn execute_inner(
        &self,
        task_model: &task::Model,
        run_id: i64,
    ) -> anyhow::Result<RunStats> {
        let dt: DataType = task_model.task_type.parse()?;
        let source = self.registry.get(&task_model.provider)?;

        // 校验数据源是否支持该数据类型
        if !source.capabilities().contains(&dt) {
            anyhow::bail!("data source '{}' does not support {}", source.name(), dt);
        }

        // 查找 Collector
        let collector = self
            .collectors
            .get(&dt)
            .ok_or_else(|| anyhow::anyhow!("no collector registered for {}", dt))?;

        // 构造上下文（进度更新和取消检查通过闭包传递）
        let db = self.db.clone();

        // 进度更新：spawn 异步任务执行（闭包是 sync 的）
        let db_for_progress = self.db.clone();
        let update_progress: Arc<dyn Fn(usize, usize) + Send + Sync> =
            Arc::new(move |total, processed| {
                let db = db_for_progress.clone();
                tokio::spawn(async move {
                    let _ = TaskService::update_progress(&db, run_id, total, processed).await;
                });
            });

        let db_for_cancel = self.db.clone();
        let is_cancelled: Arc<dyn Fn() -> bool + Send + Sync> = Arc::new(move || {
            // 同步检查无法直接调 async，用 try_block 方式
            // 简化：返回 false，取消检查由 Collector 通过其他方式
            // 实际方案：用 tokio::task::block_in_place + Handle::current().block_on
            let db = db_for_cancel.clone();
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(TaskService::is_cancel_requested(&db, run_id))
                    .unwrap_or(false)
            })
        });

        let ctx = CollectContext {
            db,
            source,
            params: task_model.params.clone(),
            run_id,
            update_progress,
            is_cancelled,
        };

        collector.collect(&ctx).await
    }
}
