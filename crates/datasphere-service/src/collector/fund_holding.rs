use crate::collector::{CollectContext, Collector};
use async_trait::async_trait;
use datasphere_core::domain::{
    DataType, FetchFundHoldingParams, FetchParams, FetchResult, RunStats,
};

pub struct FundHoldingCollector;

#[async_trait]
impl Collector for FundHoldingCollector {
    fn data_type(&self) -> DataType {
        DataType::FundHolding
    }

    async fn collect(&self, ctx: &CollectContext) -> anyhow::Result<RunStats> {
        let mut stats = RunStats::default();
        let params: FetchFundHoldingParams = ctx
            .params
            .as_ref()
            .map(|v| serde_json::from_value(v.clone()))
            .transpose()?
            .unwrap_or_default();

        // codes 为空时取全市场基金列表
        let default_codes = match crate::fund_service::FundService::list_all_codes(&ctx.db).await {
            Ok(c) => c,
            Err(e) => {
                stats.record_failure(format!("load fund codes: {e:#}"));
                return Ok(stats);
            }
        };
        let codes: Vec<String> = if params.codes.is_empty() {
            default_codes
        } else {
            params.codes.clone()
        };
        let total = codes.len();
        (ctx.update_progress)(total, 0);

        let report_date = params
            .report_date
            .unwrap_or_else(|| chrono::Local::now().date_naive());

        for (i, fund_code) in codes.iter().enumerate() {
            if (ctx.is_cancelled)() {
                tracing::info!("fund holding cancelled at {}/{}", i, total);
                break;
            }
            tracing::info!(
                "Fetching fund holdings [{}/{}] fund_code={} report_date={}",
                i + 1,
                total,
                fund_code,
                report_date
            );
            let fetch_params = FetchParams {
                data_type: DataType::FundHolding,
                params: serde_json::json!({
                    "codes": [fund_code],
                    "report_date": report_date,
                }),
            };
            match ctx.source.fetch(&fetch_params).await {
                Ok(FetchResult::FundHoldings(holdings)) => {
                    match crate::fund_holding_service::FundHoldingService::upsert_many(
                        &ctx.db, &holdings,
                    )
                    .await
                    {
                        Ok(n) => stats.record_success(n),
                        Err(e) => stats.record_failure(format!(
                            "upsert fund holdings code={}: {e:#}",
                            fund_code
                        )),
                    }
                }
                Ok(_) => stats.record_failure("unexpected fetch result type"),
                Err(e) => {
                    stats.record_failure(format!("fetch fund holdings code={}: {e:#}", fund_code))
                }
            }
            (ctx.update_progress)(total, i + 1);
        }
        Ok(stats)
    }
}
