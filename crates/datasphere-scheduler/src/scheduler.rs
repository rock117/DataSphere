//! 基于 tokio-cron-scheduler 的任务调度器。
//!
//! 支持：
//! - 启动时从 DB 加载 enabled 且有 cron 表达式的任务
//! - 动态添加 / 删除 / 启停任务
//! - cron 到点时触发 TaskRunner 执行

use datasphere_core::domain::TriggerType;
use datasphere_entity::task;
use datasphere_service::{runner::TaskRunner, task_service::TaskService};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};

pub struct Scheduler {
    sched: JobScheduler,
    /// uuid -> task_id 映射，用于动态删除
    job_map: dashmap::DashMap<i64, uuid::Uuid>,
}

impl Scheduler {
    /// 创建调度器（不启动）
    pub async fn new() -> Result<Self, JobSchedulerError> {
        let sched = JobScheduler::new().await?;
        Ok(Self {
            sched,
            job_map: dashmap::DashMap::new(),
        })
    }

    /// 启动调度器并从 DB 加载所有 enabled 且有 cron 的任务
    pub async fn start(
        &self,
        db: DatabaseConnection,
        runner: Arc<TaskRunner>,
    ) -> anyhow::Result<()> {
        self.sched.start().await?;

        let tasks = TaskService::list_enabled(&db).await?;
        let count = tasks.len();
        for t in &tasks {
            self.add_job(t, db.clone(), Arc::clone(&runner)).await?;
        }
        tracing::info!("Scheduler started with {} jobs", count);
        Ok(())
    }

    /// 添加一个 cron 任务到调度器
    pub async fn add_job(
        &self,
        task_model: &task::Model,
        db: DatabaseConnection,
        runner: Arc<TaskRunner>,
    ) -> anyhow::Result<()> {
        let Some(cron) = &task_model.cron else {
            anyhow::bail!("task {} has no cron expression", task_model.id);
        };

        let task_id = task_model.id;
        let task_name = task_model.name.clone();
        let cron_str = cron.clone();

        // 校验 cron 表达式
        let job = Job::new_async(cron_str.as_str(), move |_uuid, _l| {
            let db = db.clone();
            let runner = Arc::clone(&runner);
            let task_name = task_name.clone();
            Box::pin(async move {
                tracing::info!("[Cron] triggering task_id={} name={}", task_id, task_name);
                // 从 DB 重新读取任务，获取最新 params
                match TaskService::find_by_id(&db, task_id).await {
                    Ok(Some(t)) => {
                        if let Err(e) = runner.run(&t, TriggerType::Cron).await {
                            tracing::error!("[Cron] task {} failed: {e:#}", task_id);
                        }
                    }
                    Ok(None) => {
                        tracing::warn!("[Cron] task {} not found, may have been deleted", task_id);
                    }
                    Err(e) => {
                        tracing::error!("[Cron] failed to load task {}: {e:#}", task_id);
                    }
                }
            })
        })
        .map_err(|e| anyhow::anyhow!("invalid cron '{cron_str}': {e}"))?;

        let uuid = self.sched.add(job).await?;
        self.job_map.insert(task_id, uuid);
        tracing::info!("Added cron job: task_id={} cron={}", task_id, cron_str);
        Ok(())
    }

    /// 从调度器移除任务
    pub async fn remove_job(&self, task_id: i64) -> anyhow::Result<()> {
        if let Some((_, uuid)) = self.job_map.remove(&task_id) {
            self.sched.remove(&uuid).await?;
            tracing::info!("Removed cron job: task_id={}", task_id);
        }
        Ok(())
    }

    /// 停止调度器
    pub async fn shutdown(&mut self) -> Result<(), JobSchedulerError> {
        self.sched.shutdown().await
    }
}
