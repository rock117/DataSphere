use crate::response::{ApiResponse, Paginated};
use crate::state::AppState;
use datasphere_entity::stock;
use rocket::get;
use rocket::State;

/// 分页查询股票列表
/// GET /api/stocks?page=1&per_page=20&q=浦发&industry=银行
#[get("/stocks?<page>&<per_page>&<q>&<industry>")]
pub async fn list_stocks(
    state: &State<AppState>,
    page: Option<u64>,
    per_page: Option<u64>,
    q: Option<String>,
    industry: Option<String>,
) -> ApiResponse<Paginated<stock::Model>> {
    let page = page.unwrap_or(1).max(1);
    let per_page = per_page.unwrap_or(20).clamp(1, 200);
    let q = q.as_deref();
    let industry = industry.as_deref();

    match datasphere_service::stock_service::StockService::paginate(
        &state.db, page, per_page, q, industry,
    )
    .await
    {
        Ok((items, total)) => ApiResponse::ok(Paginated {
            items,
            total,
            page,
            per_page,
        }),
        Err(e) => ApiResponse::err(format!("{e:#}")),
    }
}

/// 获取所有行业列表
/// GET /api/industries
#[get("/industries")]
pub async fn list_industries(state: &State<AppState>) -> ApiResponse<Vec<String>> {
    match datasphere_service::stock_service::StockService::list_industries(&state.db).await {
        Ok(list) => ApiResponse::ok(list),
        Err(e) => ApiResponse::err(format!("{e:#}")),
    }
}

/// 按代码查单只股票
/// GET /api/stocks/:code
#[get("/stocks/<code>")]
pub async fn get_stock(state: &State<AppState>, code: &str) -> ApiResponse<Option<stock::Model>> {
    match datasphere_service::stock_service::StockService::find_by_code(&state.db, code).await {
        Ok(m) => ApiResponse::ok(m),
        Err(e) => ApiResponse::err(format!("{e:#}")),
    }
}
