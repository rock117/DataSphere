use datasphere_core::DataSourceRegistry;
use datasphere_scheduler::Scheduler;
use datasphere_service::collector::CollectorRegistry;
use datasphere_service::runner::TaskRunner;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

/// 全局应用状态，通过 Rocket State 注入到各路由
pub struct AppState {
    pub db: DatabaseConnection,
    pub registry: Arc<DataSourceRegistry>,
    pub collectors: Arc<CollectorRegistry>,
    pub runner: Arc<TaskRunner>,
    pub scheduler: Arc<tokio::sync::Mutex<Scheduler>>,
}

impl AppState {
    pub async fn new(
        db: DatabaseConnection,
        registry: Arc<DataSourceRegistry>,
        collectors: Arc<CollectorRegistry>,
    ) -> anyhow::Result<Self> {
        let runner = Arc::new(TaskRunner::new(
            db.clone(),
            Arc::clone(&registry),
            Arc::clone(&collectors),
        ));
        let scheduler = Arc::new(tokio::sync::Mutex::new(Scheduler::new().await?));
        Ok(Self {
            db,
            registry,
            collectors,
            runner,
            scheduler,
        })
    }
}
