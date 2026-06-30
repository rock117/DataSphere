use crate::collector::{CollectContext, Collector};
use async_trait::async_trait;
use datasphere_core::domain::{DataType, FetchIndustryParams, FetchParams, FetchResult, RunStats};

pub struct IndustryCollector;

#[async_trait]
impl Collector for IndustryCollector {
    fn data_type(&self) -> DataType {
        DataType::Industry
    }

    async fn collect(&self, ctx: &CollectContext) -> anyhow::Result<RunStats> {
        let mut stats = RunStats::default();
        let params = ctx.params.as_ref().map(|v| v.clone()).unwrap_or_default();
        let _typed: FetchIndustryParams = serde_json::from_value(params).unwrap_or_default();

        (ctx.update_progress)(1, 0);
        if (ctx.is_cancelled)() {
            return Ok(stats);
        }

        let fetch_params = FetchParams {
            data_type: DataType::Industry,
            params: ctx.params.clone().unwrap_or_default(),
        };
        match ctx.source.fetch(&fetch_params).await {
            Ok(FetchResult::Industries(items)) => {
                match crate::stock_service::StockService::update_industries(&ctx.db, &items).await {
                    Ok(n) => stats.record_success(n),
                    Err(e) => stats.record_failure(format!("update industries: {e:#}")),
                }
            }
            Ok(_) => stats.record_failure("unexpected fetch result type"),
            Err(e) => stats.record_failure(format!("fetch industries: {e:#}")),
        }
        (ctx.update_progress)(1, 1);
        Ok(stats)
    }
}
