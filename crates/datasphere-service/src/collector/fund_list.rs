use crate::collector::{CollectContext, Collector};
use async_trait::async_trait;
use datasphere_core::domain::{DataType, FetchFundListParams, FetchParams, FetchResult, RunStats};

pub struct FundListCollector;

#[async_trait]
impl Collector for FundListCollector {
    fn data_type(&self) -> DataType {
        DataType::FundList
    }

    async fn collect(&self, ctx: &CollectContext) -> anyhow::Result<RunStats> {
        let mut stats = RunStats::default();
        let params = ctx.params.as_ref().map(|v| v.clone()).unwrap_or_default();
        let _typed: FetchFundListParams = serde_json::from_value(params).unwrap_or_default();

        (ctx.update_progress)(1, 0);
        if (ctx.is_cancelled)() {
            return Ok(stats);
        }

        let fetch_params = FetchParams {
            data_type: DataType::FundList,
            params: ctx.params.clone().unwrap_or_default(),
        };
        match ctx.source.fetch(&fetch_params).await {
            Ok(FetchResult::Funds(quotes)) => {
                match crate::fund_service::FundService::upsert_many(&ctx.db, &quotes).await {
                    Ok(n) => stats.record_success(n),
                    Err(e) => stats.record_failure(format!("upsert fund list: {e:#}")),
                }
            }
            Ok(_) => stats.record_failure("unexpected fetch result type"),
            Err(e) => stats.record_failure(format!("fetch fund list: {e:#}")),
        }
        (ctx.update_progress)(1, 1);
        Ok(stats)
    }
}
