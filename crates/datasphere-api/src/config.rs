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
    pub url: String,
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
    /// 覆盖规则（环境变量优先级高于配置文件，便于在 .env 中覆盖敏感项）：
    /// - `DATABASE_URL`：覆盖 `database.url`
    pub fn load() -> anyhow::Result<Self> {
        let path = std::env::var("DATASPHERE_CONFIG").unwrap_or_else(|_| "config.toml".to_string());
        let content = std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("failed to read config file '{path}': {e}"))?;
        let mut config: AppConfig =
            toml::from_str(&content).map_err(|e| anyhow::anyhow!("failed to parse config: {e}"))?;

        // 环境变量覆盖
        if let Ok(url) = std::env::var("DATABASE_URL") {
            if !url.is_empty() {
                config.database.url = url;
            }
        }

        Ok(config)
    }
}
