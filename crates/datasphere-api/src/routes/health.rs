use crate::response::ApiResponse;
use crate::state::AppState;
use datasphere_core::domain::DataType;
use rocket::get;
use rocket::State;

#[get("/health")]
pub fn health() -> ApiResponse<&'static str> {
    ApiResponse::ok("ok")
}

/// 列出所有已注册的数据源 provider
/// GET /api/datasources?capability=stock_list  → 仅列出支持指定数据类型的
/// GET /api/datasources                         → 列出全部
#[get("/datasources?<capability>")]
pub fn list_datasources(
    state: &State<AppState>,
    capability: Option<String>,
) -> ApiResponse<Vec<String>> {
    if let Some(cap) = capability {
        match cap.parse::<DataType>() {
            Ok(dt) => ApiResponse::ok(state.registry.list_by_capability(&dt)),
            Err(e) => ApiResponse::err(format!("invalid capability: {e}")),
        }
    } else {
        ApiResponse::ok(state.registry.list())
    }
}

/// 列出所有已注册的数据类型（Collector）
/// GET /api/data_types
#[get("/data_types")]
pub fn list_data_types(state: &State<AppState>) -> ApiResponse<Vec<String>> {
    ApiResponse::ok(
        state
            .collectors
            .list()
            .into_iter()
            .map(|dt| dt.to_string())
            .collect(),
    )
}
