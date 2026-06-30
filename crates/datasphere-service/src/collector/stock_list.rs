use crate::collector::{CollectContext, Collector};
use async_trait::async_trait;
use datasphere_core::domain::{DataType, FetchParams, FetchResult, FetchStockListParams, RunStats};

pub struct StockListCollector;

#[async_trait]
impl Collector for StockListCollector {
    fn data_type(&self) -> DataType {
        DataType::StockList
    }

    async fn collect(&self, ctx: &CollectContext) -> anyhow::Result<RunStats> {
        let mut stats = RunStats::default();
        let params = ctx.params.as_ref().map(|v| v.clone()).unwrap_or_default();
        let _typed: FetchStockListParams = serde_json::from_value(params).unwrap_or_default();

        (ctx.update_progress)(1, 0);
        if (ctx.is_cancelled)() {
            return Ok(stats);
        }

        let fetch_params = FetchParams {
            data_type: DataType::StockList,
            params: ctx.params.clone().unwrap_or_default(),
        };
        match ctx.source.fetch(&fetch_params).await {
            Ok(FetchResult::Stocks(quotes)) => {
                match crate::stock_service::StockService::upsert_many(&ctx.db, &quotes).await {
                    Ok(n) => stats.record_success(n),
                    Err(e) => stats.record_failure(format!("upsert stock list: {e:#}")),
                }
            }
            Ok(_) => stats.record_failure("unexpected fetch result type"),
            Err(e) => stats.record_failure(format!("fetch stock list: {e:#}")),
        }
        (ctx.update_progress)(1, 1);
        Ok(stats)
    }
}
