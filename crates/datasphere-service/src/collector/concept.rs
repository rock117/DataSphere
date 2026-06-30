use crate::collector::{CollectContext, Collector};
use async_trait::async_trait;
use datasphere_core::domain::{DataType, FetchConceptParams, FetchParams, FetchResult, RunStats};

pub struct ConceptCollector;

#[async_trait]
impl Collector for ConceptCollector {
    fn data_type(&self) -> DataType {
        DataType::Concept
    }

    async fn collect(&self, ctx: &CollectContext) -> anyhow::Result<RunStats> {
        let mut stats = RunStats::default();
        let params = ctx.params.as_ref().map(|v| v.clone()).unwrap_or_default();
        let _typed: FetchConceptParams = serde_json::from_value(params).unwrap_or_default();

        (ctx.update_progress)(1, 0);
        if (ctx.is_cancelled)() {
            return Ok(stats);
        }

        let fetch_params = FetchParams {
            data_type: DataType::Concept,
            params: ctx.params.clone().unwrap_or_default(),
        };
        match ctx.source.fetch(&fetch_params).await {
            Ok(FetchResult::Concepts(concepts, stock_concepts)) => {
                match crate::concept_service::ConceptService::upsert_all(
                    &ctx.db,
                    &concepts,
                    &stock_concepts,
                )
                .await
                {
                    Ok(n) => stats.record_success(n),
                    Err(e) => stats.record_failure(format!("upsert concepts: {e:#}")),
                }
            }
            Ok(_) => stats.record_failure("unexpected fetch result type"),
            Err(e) => stats.record_failure(format!("fetch concepts: {e:#}")),
        }
        (ctx.update_progress)(1, 1);
        Ok(stats)
    }
}
