use crate::response::ApiResponse;
use crate::state::AppState;
use datasphere_entity::{concept, stock};
use rocket::get;
use rocket::State;

/// 列出所有概念板块
/// GET /api/concepts
#[get("/concepts")]
pub async fn list_concepts(state: &State<AppState>) -> ApiResponse<Vec<concept::Model>> {
    match datasphere_service::concept_service::ConceptService::list_all(&state.db).await {
        Ok(list) => ApiResponse::ok(list),
        Err(e) => ApiResponse::err(format!("{e:#}")),
    }
}

/// 查询某概念的成分股
/// GET /api/concepts/:id/stocks
#[get("/concepts/<id>/stocks")]
pub async fn list_concept_stocks(
    state: &State<AppState>,
    id: i64,
) -> ApiResponse<Vec<stock::Model>> {
    match datasphere_service::concept_service::ConceptService::list_stocks_by_concept(&state.db, id)
        .await
    {
        Ok(codes) => {
            // codes -> stock models
            let mut stocks = Vec::new();
            for code in codes {
                if let Ok(Some(s)) =
                    datasphere_service::stock_service::StockService::find_by_code(&state.db, &code)
                        .await
                {
                    stocks.push(s);
                }
            }
            ApiResponse::ok(stocks)
        }
        Err(e) => ApiResponse::err(format!("{e:#}")),
    }
}

/// 查询某股票所属的所有概念
/// GET /api/stocks/:code/concepts
#[get("/stocks/<code>/concepts")]
pub async fn list_stock_concepts(
    state: &State<AppState>,
    code: &str,
) -> ApiResponse<Vec<concept::Model>> {
    match datasphere_service::concept_service::ConceptService::list_concepts_by_stock(
        &state.db, code,
    )
    .await
    {
        Ok(list) => ApiResponse::ok(list),
        Err(e) => ApiResponse::err(format!("{e:#}")),
    }
}
