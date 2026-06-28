use crate::response::{ApiResponse, Paginated};
use crate::state::AppState;
use datasphere_entity::fund;
use rocket::get;
use rocket::State;

/// 分页查询基金列表
/// GET /api/funds?page=1&per_page=20&q=华夏
#[get("/funds?<page>&<per_page>&<q>")]
pub async fn list_funds(
    state: &State<AppState>,
    page: Option<u64>,
    per_page: Option<u64>,
    q: Option<String>,
) -> ApiResponse<Paginated<fund::Model>> {
    let page = page.unwrap_or(1).max(1);
    let per_page = per_page.unwrap_or(20).clamp(1, 200);
    let q = q.as_deref();

    match datasphere_service::fund_service::FundService::paginate(&state.db, page, per_page, q)
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

/// 按代码查单只基金
/// GET /api/funds/:code
#[get("/funds/<code>")]
pub async fn get_fund(state: &State<AppState>, code: &str) -> ApiResponse<Option<fund::Model>> {
    match datasphere_service::fund_service::FundService::find_by_code(&state.db, code).await {
        Ok(m) => ApiResponse::ok(m),
        Err(e) => ApiResponse::err(format!("{e:#}")),
    }
}
