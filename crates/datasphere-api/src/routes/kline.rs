use crate::response::ApiResponse;
use crate::state::AppState;
use datasphere_entity::kline;
use rocket::get;
use rocket::State;

/// 查询某股票在日期范围内的日K
/// GET /api/klines/:code?start=2024-01-01&end=2024-06-01
#[get("/klines/<code>?<start>&<end>")]
pub async fn get_klines(
    state: &State<AppState>,
    code: &str,
    start: Option<String>,
    end: Option<String>,
) -> ApiResponse<Vec<kline::Model>> {
    let start = match start {
        Some(s) => match chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d") {
            Ok(d) => Some(d),
            Err(_) => return ApiResponse::err(format!("invalid start date: {s}")),
        },
        None => None,
    };
    let end = match end {
        Some(s) => match chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d") {
            Ok(d) => Some(d),
            Err(_) => return ApiResponse::err(format!("invalid end date: {s}")),
        },
        None => None,
    };

    match datasphere_service::kline_service::KlineService::query(&state.db, code, start, end).await
    {
        Ok(rows) => ApiResponse::ok(rows),
        Err(e) => ApiResponse::err(format!("{e:#}")),
    }
}
