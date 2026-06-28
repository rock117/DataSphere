use crate::datasource::DataSource;
use crate::error::{CoreError, Result};
use dashmap::DashMap;
use std::sync::Arc;

/// 数据源注册表。按 provider 名注册与查找 `DataSource` 实现。
#[derive(Default)]
pub struct DataSourceRegistry {
    sources: DashMap<String, Arc<dyn DataSource>>,
}

impl DataSourceRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册数据源
    pub fn register(&self, source: Arc<dyn DataSource>) {
        let name = source.name().to_string();
        self.sources.insert(name, source);
    }

    /// 按 provider 名查找数据源
    pub fn get(&self, provider: &str) -> Result<Arc<dyn DataSource>> {
        self.sources
            .get(provider)
            .map(|e| Arc::clone(&e))
            .ok_or_else(|| CoreError::DataSourceNotFound(provider.to_string()))
    }

    /// 列出所有已注册的 provider 名
    pub fn list(&self) -> Vec<String> {
        self.sources.iter().map(|e| e.key().clone()).collect()
    }
}
