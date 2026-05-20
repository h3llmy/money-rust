use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug)]
pub struct AppError {
    pub status: StatusCode,
    pub message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = Json(ErrorResponse {
            error: self.message,
        });
        (self.status, body).into_response()
    }
}

impl From<(StatusCode, String)> for AppError {
    fn from(err: (StatusCode, String)) -> Self {
        Self {
            status: err.0,
            message: err.1,
        }
    }
}

impl From<(StatusCode, &'static str)> for AppError {
    fn from(err: (StatusCode, &'static str)) -> Self {
        Self {
            status: err.0,
            message: err.1.to_string(),
        }
    }
}
