use crate::response::ApiResponse;
use crate::state::AppState;
use rocket::get;
use rocket::State;

#[get("/health")]
pub fn health() -> ApiResponse<&'static str> {
    ApiResponse::ok("ok")
}

/// 列出所有已注册的数据源 provider
/// GET /api/datasources
#[get("/datasources")]
pub fn list_datasources(state: &State<AppState>) -> ApiResponse<Vec<String>> {
    ApiResponse::ok(state.registry.list())
}
