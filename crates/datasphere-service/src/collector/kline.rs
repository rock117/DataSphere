use crate::collector::{CollectContext, Collector};
use async_trait::async_trait;
use datasphere_core::domain::{DataType, FetchKlineParams, FetchParams, FetchResult, RunStats};

pub struct KlineCollector;

#[async_trait]
impl Collector for KlineCollector {
    fn data_type(&self) -> DataType {
        DataType::Kline
    }

    async fn collect(&self, ctx: &CollectContext) -> anyhow::Result<RunStats> {
        let mut stats = RunStats::default();
        let params: FetchKlineParams = ctx
            .params
            .as_ref()
            .map(|v| serde_json::from_value(v.clone()))
            .transpose()?
            .unwrap_or_default();

        // codes 为空时取全市场股票列表
        let default_codes = match crate::stock_service::StockService::list_all_codes(&ctx.db).await
        {
            Ok(c) => c,
            Err(e) => {
                stats.record_failure(format!("load stock codes: {e:#}"));
                return Ok(stats);
            }
        };
        let requests = params.to_requests(&default_codes);
        let total = requests.len();
        (ctx.update_progress)(total, 0);

        for (i, req) in requests.iter().enumerate() {
            if (ctx.is_cancelled)() {
                tracing::info!("kline cancelled at {}/{}", i, total);
                break;
            }
            tracing::info!(
                "Fetching kline [{}/{}] code={} start={} end={}",
                i + 1,
                total,
                req.code,
                req.start,
                req.end
            );
            let fetch_params = FetchParams {
                data_type: DataType::Kline,
                params: serde_json::to_value(req).unwrap_or_default(),
            };
            match ctx.source.fetch(&fetch_params).await {
                Ok(FetchResult::Klines(quotes)) => {
                    match crate::kline_service::KlineService::upsert_many(&ctx.db, &quotes).await {
                        Ok(n) => stats.record_success(n),
                        Err(e) => {
                            stats.record_failure(format!("upsert kline code={}: {e:#}", req.code))
                        }
                    }
                }
                Ok(_) => stats.record_failure("unexpected fetch result type"),
                Err(e) => stats.record_failure(format!("fetch kline code={}: {e:#}", req.code)),
            }
            (ctx.update_progress)(total, i + 1);
        }
        Ok(stats)
    }
}
