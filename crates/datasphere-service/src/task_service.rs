use datasphere_core::domain::{RunStatus, TaskType, TriggerType};
use datasphere_entity::{task, task_run};
use sea_orm::entity::prelude::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};

pub struct TaskService;

/// 任务 + 最近一次运行摘要（用于任务列表展示）
#[derive(Debug, Clone, serde::Serialize)]
pub struct TaskWithLastRun {
    #[serde(flatten)]
    pub task: task::Model,
    pub last_run_id: Option<i64>,
    pub last_run_status: Option<String>,
    pub last_run_at: Option<chrono::NaiveDateTime>,
    pub last_run_success: Option<i64>,
    pub last_run_failed: Option<i64>,
}

/// 创建任务的输入
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateTaskInput {
    pub name: String,
    pub task_type: String,
    pub provider: String,
    pub cron: Option<String>,
    pub params: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}

/// 更新任务的输入
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateTaskInput {
    pub name: Option<String>,
    pub cron: Option<String>,
    pub params: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}

impl TaskService {
    /// 创建任务
    pub async fn create(
        db: &DatabaseConnection,
        input: CreateTaskInput,
    ) -> anyhow::Result<task::Model> {
        // 校验 task_type 合法
        let _tt: TaskType = input.task_type.parse()?;
        let am = task::ActiveModel {
            name: Set(input.name),
            task_type: Set(input.task_type),
            provider: Set(input.provider),
            cron: Set(input.cron),
            params: Set(input.params),
            enabled: Set(input.enabled.unwrap_or(true)),
            ..Default::default()
        };
        let model = am.insert(db).await?;
        Ok(model)
    }

    /// 更新任务
    pub async fn update(
        db: &DatabaseConnection,
        id: i64,
        input: UpdateTaskInput,
    ) -> anyhow::Result<Option<task::Model>> {
        let existing = task::Entity::find_by_id(id).one(db).await?;
        let Some(m) = existing else {
            return Ok(None);
        };
        let mut am: task::ActiveModel = m.into();
        if let Some(name) = input.name {
            am.name = Set(name);
        }
        if let Some(cron) = input.cron {
            am.cron = Set(Some(cron));
        }
        if let Some(params) = input.params {
            am.params = Set(Some(params));
        }
        if let Some(enabled) = input.enabled {
            am.enabled = Set(enabled);
        }
        let model = am.update(db).await?;
        Ok(Some(model))
    }

    /// 删除任务
    pub async fn delete(db: &DatabaseConnection, id: i64) -> anyhow::Result<bool> {
        let res = task::Entity::delete_by_id(id).exec(db).await?;
        Ok(res.rows_affected > 0)
    }

    /// 按 id 查任务
    pub async fn find_by_id(
        db: &DatabaseConnection,
        id: i64,
    ) -> anyhow::Result<Option<task::Model>> {
        task::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(Into::into)
    }

    /// 列出所有任务
    pub async fn list_all(db: &DatabaseConnection) -> anyhow::Result<Vec<task::Model>> {
        task::Entity::find()
            .order_by_desc(task::Column::CreatedAt)
            .all(db)
            .await
            .map_err(Into::into)
    }

    /// 列出所有任务，附带最近一次运行摘要
    pub async fn list_with_last_run(
        db: &DatabaseConnection,
    ) -> anyhow::Result<Vec<TaskWithLastRun>> {
        let tasks = task::Entity::find()
            .order_by_desc(task::Column::CreatedAt)
            .all(db)
            .await?;

        let mut out = Vec::with_capacity(tasks.len());
        for t in tasks {
            let last = task_run::Entity::find()
                .filter(task_run::Column::TaskId.eq(t.id))
                .order_by_desc(task_run::Column::StartedAt)
                .one(db)
                .await?;
            out.push(TaskWithLastRun {
                last_run_id: last.as_ref().map(|r| r.id),
                last_run_status: last.as_ref().map(|r| r.status.clone()),
                last_run_at: last.as_ref().map(|r| r.started_at),
                last_run_success: last.as_ref().map(|r| r.success_count),
                last_run_failed: last.as_ref().map(|r| r.failed_count),
                task: t,
            });
        }
        Ok(out)
    }

    /// 列出所有启用的任务（调度器启动时用）
    pub async fn list_enabled(db: &DatabaseConnection) -> anyhow::Result<Vec<task::Model>> {
        task::Entity::find()
            .filter(task::Column::Enabled.eq(true))
            .filter(task::Column::Cron.is_not_null())
            .all(db)
            .await
            .map_err(Into::into)
    }

    // ---- task_runs ----

    /// 创建一条运行记录
    pub async fn create_run(
        db: &DatabaseConnection,
        task_id: i64,
        trigger: TriggerType,
    ) -> anyhow::Result<task_run::Model> {
        let now = chrono::Local::now().naive_local();
        let am = task_run::ActiveModel {
            task_id: Set(task_id),
            status: Set(RunStatus::Running.to_string()),
            trigger_type: Set(trigger.to_string()),
            started_at: Set(now),
            ..Default::default()
        };
        am.insert(db).await.map_err(Into::into)
    }

    /// 完成运行记录
    pub async fn finish_run(
        db: &DatabaseConnection,
        run_id: i64,
        status: RunStatus,
        stats: &datasphere_core::domain::RunStats,
        error: Option<String>,
    ) -> anyhow::Result<()> {
        let now = chrono::Local::now().naive_local();
        let existing = task_run::Entity::find_by_id(run_id).one(db).await?;
        if let Some(m) = existing {
            let mut am: task_run::ActiveModel = m.into();
            am.status = Set(status.to_string());
            am.finished_at = Set(Some(now));
            am.records_affected = Set(stats.total() as i64);
            am.success_count = Set(stats.success as i64);
            am.failed_count = Set(stats.failed as i64);
            am.duration_ms = Set(stats.duration_ms as i64);
            am.processed = Set(stats.total() as i64);
            am.error = Set(error);
            am.update(db).await?;
        }
        Ok(())
    }

    /// 更新运行进度（total / processed），不改变状态
    pub async fn update_progress(
        db: &DatabaseConnection,
        run_id: i64,
        total: usize,
        processed: usize,
    ) -> anyhow::Result<()> {
        let existing = task_run::Entity::find_by_id(run_id).one(db).await?;
        if let Some(m) = existing {
            let mut am: task_run::ActiveModel = m.into();
            am.total = Set(total as i64);
            am.processed = Set(processed as i64);
            am.update(db).await?;
        }
        Ok(())
    }

    /// 标记某次运行为"请求取消"。runner 循环检查到后提前结束。
    /// 仅对 Running 状态的 run 有效。
    pub async fn request_cancel(db: &DatabaseConnection, run_id: i64) -> anyhow::Result<bool> {
        let existing = task_run::Entity::find_by_id(run_id).one(db).await?;
        let Some(m) = existing else { return Ok(false) };
        if m.status != RunStatus::Running.to_string() {
            return Ok(false);
        }
        let mut am: task_run::ActiveModel = m.into();
        am.cancel_requested = Set(true);
        am.update(db).await?;
        Ok(true)
    }

    /// 查询某次运行是否被请求取消
    pub async fn is_cancel_requested(db: &DatabaseConnection, run_id: i64) -> anyhow::Result<bool> {
        let existing = task_run::Entity::find_by_id(run_id).one(db).await?;
        Ok(existing.map(|m| m.cancel_requested).unwrap_or(false))
    }

    /// 查询某任务的执行历史
    pub async fn list_runs(
        db: &DatabaseConnection,
        task_id: i64,
        limit: u64,
    ) -> anyhow::Result<Vec<task_run::Model>> {
        task_run::Entity::find()
            .filter(task_run::Column::TaskId.eq(task_id))
            .order_by_desc(task_run::Column::StartedAt)
            .limit(limit)
            .all(db)
            .await
            .map_err(Into::into)
    }

    /// 查单条运行记录
    pub async fn find_run(
        db: &DatabaseConnection,
        run_id: i64,
    ) -> anyhow::Result<Option<task_run::Model>> {
        task_run::Entity::find_by_id(run_id)
            .one(db)
            .await
            .map_err(Into::into)
    }
}
