mod config;
mod response;
mod routes;
mod state;

use datasphere_core::{DataSourceRegistry, MockDataSource};
use datasphere_service::collector::{
    CollectorRegistry, ConceptCollector, FundHoldingCollector, FundListCollector,
    IndustryCollector, KlineCollector, StockListCollector,
};
use sea_orm::{ConnectOptions, Database};
use std::sync::Arc;

use routes::concept::{list_concept_stocks, list_concepts, list_stock_concepts};
use routes::fund::{
    get_fund, list_fund_holdings, list_fund_holdings_by_date, list_funds, list_report_dates,
};
use routes::health;
use routes::health::{list_data_types, list_datasources};
use routes::kline::get_klines;
use routes::stock::{get_stock, list_industries, list_stocks};
use routes::task::*;
use state::AppState;

#[rocket::launch]
async fn rocket() -> _ {
    // 加载 .env（不存在则忽略，生产环境可用纯环境变量）
    let _ = dotenvy::dotenv();

    // 初始化日志（RUST_LOG 可来自 .env 或系统环境变量）
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "datasphere=info,rocket=info".into()),
        )
        .init();

    // 加载配置
    let config = config::AppConfig::load().expect("failed to load config");
    tracing::info!(
        "Config loaded: server={}:{}",
        config.server.host,
        config.server.port
    );
    tracing::info!(
        "Datasource default_provider={}, scheduler timezone={}",
        config.datasource.default_provider,
        config.scheduler.timezone
    );

    // 连接数据库（连接串来自 .env 的 DATABASE_URL）
    let db_url = config.database_url().expect("DATABASE_URL not configured");
    let mut opt = ConnectOptions::new(&db_url);
    opt.max_connections(config.database.max_connections)
        .min_connections(config.database.min_connections);
    let db = Database::connect(opt)
        .await
        .expect("failed to connect database");
    tracing::info!("Database connected");

    // 注册数据源
    let registry = Arc::new(DataSourceRegistry::new());
    registry.register(Arc::new(MockDataSource::new()));
    tracing::info!("DataSources registered: {:?}", registry.list());

    // 注册采集器
    let mut collectors = CollectorRegistry::new();
    collectors.register(Arc::new(StockListCollector));
    collectors.register(Arc::new(IndustryCollector));
    collectors.register(Arc::new(ConceptCollector));
    collectors.register(Arc::new(FundListCollector));
    collectors.register(Arc::new(FundHoldingCollector));
    collectors.register(Arc::new(KlineCollector));
    let collectors = Arc::new(collectors);
    tracing::info!("Collectors registered: {:?}", collectors.list());

    // 创建 AppState
    let state = AppState::new(db.clone(), registry, collectors)
        .await
        .expect("failed to create AppState");

    // 启动调度器
    {
        let sched = state.scheduler.lock().await;
        sched
            .start(db, Arc::clone(&state.runner))
            .await
            .expect("failed to start scheduler");
    }

    // 构建 Rocket
    rocket::build()
        .manage(state)
        .mount(
            "/api",
            rocket::routes![
                health::health,
                list_datasources,
                list_data_types,
                list_stocks,
                get_stock,
                list_industries,
                list_concepts,
                list_concept_stocks,
                list_stock_concepts,
                list_funds,
                get_fund,
                list_fund_holdings,
                list_fund_holdings_by_date,
                list_report_dates,
                get_klines,
                list_tasks,
                get_task,
                create_task,
                update_task,
                delete_task,
                toggle_task,
                run_task,
                refetch_task,
                next_run,
                list_runs,
                get_run,
                cancel_run,
            ],
        )
        .configure(rocket::Config {
            port: config.server.port,
            address: config
                .server
                .host
                .parse()
                .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)),
            ..rocket::Config::default()
        })
}
