use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub datasource: DatasourceConfig,
    pub scheduler: SchedulerConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    /// 连接池大小等非敏感配置（在 config.toml 中）
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatasourceConfig {
    pub default_provider: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SchedulerConfig {
    pub timezone: String,
}

impl AppConfig {
    /// 从 config.toml 加载，路径优先取 DATASPHERE_CONFIG 环境变量。
    ///
    /// 数据库连接串 `DATABASE_URL` 仅从环境变量（.env）读取，不在 config.toml 中配置，
    /// 避免敏感信息进入版本库。
    pub fn load() -> anyhow::Result<Self> {
        let path = std::env::var("DATASPHERE_CONFIG").unwrap_or_else(|_| "config.toml".to_string());
        let content = std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("failed to read config file '{path}': {e}"))?;
        let config: AppConfig =
            toml::from_str(&content).map_err(|e| anyhow::anyhow!("failed to parse config: {e}"))?;
        Ok(config)
    }

    /// 数据库连接串：仅从 DATABASE_URL 环境变量读取（由 .env 提供）
    pub fn database_url(&self) -> anyhow::Result<String> {
        std::env::var("DATABASE_URL")
            .map_err(|_| anyhow::anyhow!("DATABASE_URL is not set. Please configure it in .env"))
    }
}
