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

/// 查询某基金的成分股（默认最新报告期）
/// GET /api/funds/:code/holdings?limit=20
#[get("/funds/<code>/holdings?<limit>")]
pub async fn list_fund_holdings(
    state: &State<AppState>,
    code: &str,
    limit: Option<u64>,
) -> ApiResponse<Vec<datasphere_entity::fund_holding::Model>> {
    let limit = limit.unwrap_or(20).clamp(1, 200);
    match datasphere_service::fund_holding_service::FundHoldingService::list_by_fund(
        &state.db, code, limit,
    )
    .await
    {
        Ok(rows) => ApiResponse::ok(rows),
        Err(e) => ApiResponse::err(format!("{e:#}")),
    }
}

/// 查询某基金指定报告期的成分股
/// GET /api/funds/:code/holdings/:report_date
#[get("/funds/<code>/holdings/<report_date>")]
pub async fn list_fund_holdings_by_date(
    state: &State<AppState>,
    code: &str,
    report_date: String,
) -> ApiResponse<Vec<datasphere_entity::fund_holding::Model>> {
    let date = match chrono::NaiveDate::parse_from_str(&report_date, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => return ApiResponse::err(format!("invalid date: {report_date}")),
    };
    match datasphere_service::fund_holding_service::FundHoldingService::list_by_fund_and_date(
        &state.db, code, date,
    )
    .await
    {
        Ok(rows) => ApiResponse::ok(rows),
        Err(e) => ApiResponse::err(format!("{e:#}")),
    }
}

/// 查询某基金所有报告期
/// GET /api/funds/:code/report_dates
#[get("/funds/<code>/report_dates")]
pub async fn list_report_dates(
    state: &State<AppState>,
    code: &str,
) -> ApiResponse<Vec<chrono::NaiveDate>> {
    match datasphere_service::fund_holding_service::FundHoldingService::list_report_dates(
        &state.db, code,
    )
    .await
    {
        Ok(dates) => ApiResponse::ok(dates),
        Err(e) => ApiResponse::err(format!("{e:#}")),
    }
}
