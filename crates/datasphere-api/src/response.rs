use rocket::request::Request;
use rocket::response::{self, Responder};
use rocket::serde::json::Json;
use serde::Serialize;

/// 统一 API 响应格式
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiResponse<T> {
    Ok { data: T },
    Err { error: String },
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self::Ok { data }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self::Err { error: msg.into() }
    }
}

// 让 ApiResponse 可直接作为 Rocket handler 返回类型
impl<'r, T: Serialize> Responder<'r, 'static> for ApiResponse<T> {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        Json(self).respond_to(req)
    }
}

/// 分页响应
#[derive(Debug, Serialize)]
pub struct Paginated<T: Serialize> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
}
