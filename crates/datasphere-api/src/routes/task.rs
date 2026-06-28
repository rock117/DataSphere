use crate::response::ApiResponse;
use crate::state::AppState;
use datasphere_core::domain::TriggerType;
use datasphere_entity::{task, task_run};
use datasphere_service::task_service::{CreateTaskInput, UpdateTaskInput};
use rocket::serde::json::Json;
use rocket::{delete, get, post, put, State};
use std::sync::Arc;

/// 列出所有任务（含最近一次运行摘要）
/// GET /api/tasks
#[get("/tasks")]
pub async fn list_tasks(
    state: &State<AppState>,
) -> ApiResponse<Vec<datasphere_service::task_service::TaskWithLastRun>> {
    match datasphere_service::task_service::TaskService::list_with_last_run(&state.db).await {
        Ok(tasks) => ApiResponse::ok(tasks),
        Err(e) => ApiResponse::err(format!("{e:#}")),
    }
}

/// 查单条任务
/// GET /api/tasks/:id
#[get("/tasks/<id>")]
pub async fn get_task(state: &State<AppState>, id: i64) -> ApiResponse<Option<task::Model>> {
    match datasphere_service::task_service::TaskService::find_by_id(&state.db, id).await {
        Ok(m) => ApiResponse::ok(m),
        Err(e) => ApiResponse::err(format!("{e:#}")),
    }
}

/// 创建任务
/// POST /api/tasks
#[post("/tasks", data = "<input>")]
pub async fn create_task(
    state: &State<AppState>,
    input: Json<CreateTaskInput>,
) -> ApiResponse<task::Model> {
    let input = input.into_inner();

    // 校验 cron（如有）
    if let Some(cron) = &input.cron {
        if let Err(e) = cron::validate_cron(cron) {
            return ApiResponse::err(format!("invalid cron: {e}"));
        }
    }

    match datasphere_service::task_service::TaskService::create(&state.db, input.clone()).await {
        Ok(m) => {
            // 如果有 cron 且 enabled，加入调度器
            if m.enabled && m.cron.is_some() {
                let sched = state.scheduler.lock().await;
                if let Err(e) = sched
                    .add_job(&m, state.db.clone(), Arc::clone(&state.runner))
                    .await
                {
                    tracing::error!("Failed to add job to scheduler: {e:#}");
                }
            }
            ApiResponse::ok(m)
        }
        Err(e) => ApiResponse::err(format!("{e:#}")),
    }
}

/// 更新任务
/// PUT /api/tasks/:id
#[put("/tasks/<id>", data = "<input>")]
pub async fn update_task(
    state: &State<AppState>,
    id: i64,
    input: Json<UpdateTaskInput>,
) -> ApiResponse<Option<task::Model>> {
    let input = input.into_inner();

    if let Some(cron) = &input.cron {
        if let Err(e) = cron::validate_cron(cron) {
            return ApiResponse::err(format!("invalid cron: {e}"));
        }
    }

    match datasphere_service::task_service::TaskService::update(&state.db, id, input).await {
        Ok(Some(m)) => {
            // 同步调度器：先删后加（如果 enabled 且有 cron）
            let sched = state.scheduler.lock().await;
            let _ = sched.remove_job(id).await;
            if m.enabled && m.cron.is_some() {
                if let Err(e) = sched
                    .add_job(&m, state.db.clone(), Arc::clone(&state.runner))
                    .await
                {
                    tracing::error!("Failed to re-add job to scheduler: {e:#}");
                }
            }
            ApiResponse::ok(Some(m))
        }
        Ok(None) => ApiResponse::ok(None),
        Err(e) => ApiResponse::err(format!("{e:#}")),
    }
}

/// 删除任务
/// DELETE /api/tasks/:id
#[delete("/tasks/<id>")]
pub async fn delete_task(state: &State<AppState>, id: i64) -> ApiResponse<bool> {
    // 先从调度器移除
    let sched = state.scheduler.lock().await;
    let _ = sched.remove_job(id).await;
    drop(sched);

    match datasphere_service::task_service::TaskService::delete(&state.db, id).await {
        Ok(ok) => ApiResponse::ok(ok),
        Err(e) => ApiResponse::err(format!("{e:#}")),
    }
}

/// 手动触发任务执行
/// POST /api/tasks/:id/run
#[post("/tasks/<id>/run")]
pub async fn run_task(state: &State<AppState>, id: i64) -> ApiResponse<i64> {
    let task = match datasphere_service::task_service::TaskService::find_by_id(&state.db, id).await
    {
        Ok(Some(t)) => t,
        Ok(None) => return ApiResponse::err(format!("task {id} not found")),
        Err(e) => return ApiResponse::err(format!("{e:#}")),
    };

    match state.runner.run(&task, TriggerType::Manual).await {
        Ok(run_id) => ApiResponse::ok(run_id),
        Err(e) => ApiResponse::err(format!("{e:#}")),
    }
}

/// 重新拉取（复用现有 params 再跑一次）
/// POST /api/tasks/:id/refetch
#[post("/tasks/<id>/refetch")]
pub async fn refetch_task(state: &State<AppState>, id: i64) -> ApiResponse<i64> {
    let task = match datasphere_service::task_service::TaskService::find_by_id(&state.db, id).await
    {
        Ok(Some(t)) => t,
        Ok(None) => return ApiResponse::err(format!("task {id} not found")),
        Err(e) => return ApiResponse::err(format!("{e:#}")),
    };

    match state.runner.run(&task, TriggerType::Manual).await {
        Ok(run_id) => ApiResponse::ok(run_id),
        Err(e) => ApiResponse::err(format!("{e:#}")),
    }
}

/// 查询某任务的执行历史
/// GET /api/tasks/:id/runs?limit=20
#[get("/tasks/<id>/runs?<limit>")]
pub async fn list_runs(
    state: &State<AppState>,
    id: i64,
    limit: Option<u64>,
) -> ApiResponse<Vec<task_run::Model>> {
    let limit = limit.unwrap_or(20).clamp(1, 200);
    match datasphere_service::task_service::TaskService::list_runs(&state.db, id, limit).await {
        Ok(runs) => ApiResponse::ok(runs),
        Err(e) => ApiResponse::err(format!("{e:#}")),
    }
}

/// 查单条运行记录
/// GET /api/runs/:run_id
#[get("/runs/<run_id>")]
pub async fn get_run(state: &State<AppState>, run_id: i64) -> ApiResponse<Option<task_run::Model>> {
    match datasphere_service::task_service::TaskService::find_run(&state.db, run_id).await {
        Ok(m) => ApiResponse::ok(m),
        Err(e) => ApiResponse::err(format!("{e:#}")),
    }
}

/// 切换任务启用状态（一键开关）
/// PATCH /api/tasks/:id/toggle
#[post("/tasks/<id>/toggle")]
pub async fn toggle_task(state: &State<AppState>, id: i64) -> ApiResponse<Option<task::Model>> {
    let task = match datasphere_service::task_service::TaskService::find_by_id(&state.db, id).await
    {
        Ok(Some(t)) => t,
        Ok(None) => return ApiResponse::ok(None),
        Err(e) => return ApiResponse::err(format!("{e:#}")),
    };
    let new_enabled = !task.enabled;
    let input = datasphere_service::task_service::UpdateTaskInput {
        name: None,
        cron: None,
        params: None,
        enabled: Some(new_enabled),
    };
    match datasphere_service::task_service::TaskService::update(&state.db, id, input).await {
        Ok(Some(m)) => {
            // 同步调度器：启用且有 cron -> 添加；禁用或无 cron -> 移除
            let sched = state.scheduler.lock().await;
            let _ = sched.remove_job(id).await;
            if m.enabled && m.cron.is_some() {
                if let Err(e) = sched
                    .add_job(&m, state.db.clone(), Arc::clone(&state.runner))
                    .await
                {
                    tracing::error!("Failed to add job after toggle: {e:#}");
                }
            }
            ApiResponse::ok(Some(m))
        }
        Ok(None) => ApiResponse::ok(None),
        Err(e) => ApiResponse::err(format!("{e:#}")),
    }
}

/// 请求取消某次运行
/// POST /api/runs/:run_id/cancel
#[post("/runs/<run_id>/cancel")]
pub async fn cancel_run(state: &State<AppState>, run_id: i64) -> ApiResponse<bool> {
    match datasphere_service::task_service::TaskService::request_cancel(&state.db, run_id).await {
        Ok(ok) => ApiResponse::ok(ok),
        Err(e) => ApiResponse::err(format!("{e:#}")),
    }
}

/// 预览任务的下次执行时间
/// GET /api/tasks/:id/next_run?count=5
#[get("/tasks/<id>/next_run?<count>")]
pub async fn next_run(
    state: &State<AppState>,
    id: i64,
    count: Option<u64>,
) -> ApiResponse<Vec<String>> {
    let count = count.unwrap_or(5).clamp(1, 20);
    let task = match datasphere_service::task_service::TaskService::find_by_id(&state.db, id).await
    {
        Ok(Some(t)) => t,
        Ok(None) => return ApiResponse::ok(vec![]),
        Err(e) => return ApiResponse::err(format!("{e:#}")),
    };
    let Some(cron) = &task.cron else {
        return ApiResponse::ok(vec![]);
    };
    match croner::Cron::new(cron).parse() {
        Ok(c) => {
            let mut out = Vec::new();
            let mut from = chrono::Local::now();
            for _ in 0..count {
                match c.find_next_occurrence(&from, false) {
                    Ok(next) => {
                        out.push(next.format("%Y-%m-%d %H:%M:%S").to_string());
                        from = next + chrono::Duration::seconds(1);
                    }
                    Err(_) => break,
                }
            }
            ApiResponse::ok(out)
        }
        Err(e) => ApiResponse::err(format!("invalid cron: {e}")),
    }
}

// cron 表达式校验
mod cron {
    pub fn validate_cron(expr: &str) -> Result<(), String> {
        // tokio-cron-scheduler 使用 croner，尝试构造一个 Job 来校验
        tokio_cron_scheduler::Job::new_async(expr, |_, _| Box::pin(async {}))
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}
