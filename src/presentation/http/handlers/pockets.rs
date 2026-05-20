use axum::{extract::{State, Path, Query}, Json, http::StatusCode};
use crate::shared::app_state::AppState;
use crate::shared::auth::AuthUser;
use crate::domain::pockets::dto::{PocketResponse, CreatePocketRequest, UpdatePocketRequest};
use crate::shared::pagination::PaginationQuery;
use crate::shared::response::{ApiResponse, PaginationResponse};
use crate::shared::error::AppError;
use std::sync::Arc;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/api/v1/pockets",
    params(PaginationQuery),
    responses(
        (status = 200, description = "List all pockets", body = PocketPaginationResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(
        ("BearerAuth" = [])
    ),
    tag = "Pockets"
)]
pub async fn list_pockets(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginationResponse<Vec<PocketResponse>>>, AppError> {
    let (data, total) = state.pocket_service
        .list_all_pockets(auth_user.id, pagination.clone())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let response = PaginationResponse::new(
        data.into_iter().map(PocketResponse::from).collect(),
        pagination.get_page(),
        pagination.get_limit(),
        total,
    );

    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/api/v1/pockets",
    request_body = CreatePocketRequest,
    responses(
        (status = 200, description = "Create a new pocket", body = PocketApiResponse),
        (status = 400, description = "Bad Request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(
        ("BearerAuth" = [])
    ),
    tag = "Pockets"
)]
pub async fn create_pocket(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Json(payload): Json<CreatePocketRequest>,
) -> Result<Json<ApiResponse<PocketResponse>>, AppError> {
    use std::str::FromStr;
    let balance = bigdecimal::BigDecimal::from_str(&payload.balance)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid balance: {}", e)))?;

    let result = state.pocket_service
        .create_pocket(auth_user.id, payload.name, payload.pocket_type, payload.currency, balance)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(ApiResponse { data: PocketResponse::from(result) }))
}

#[utoipa::path(
    get,
    path = "/api/v1/pockets/{id}",
    params(
        ("id" = Uuid, Path, description = "Pocket ID")
    ),
    responses(
        (status = 200, description = "Get pocket by ID", body = PocketApiResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Pocket not found", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(
        ("BearerAuth" = [])
    ),
    tag = "Pockets"
)]
pub async fn get_pocket(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<PocketResponse>>, AppError> {
    let result = state.pocket_service
        .get_pocket_by_id(id, auth_user.id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Pocket not found".to_string()))?;

    Ok(Json(ApiResponse { data: PocketResponse::from(result) }))
}

#[utoipa::path(
    put,
    path = "/api/v1/pockets/{id}",
    params(
        ("id" = Uuid, Path, description = "Pocket ID")
    ),
    request_body = UpdatePocketRequest,
    responses(
        (status = 200, description = "Update pocket", body = PocketApiResponse),
        (status = 400, description = "Bad Request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(
        ("BearerAuth" = [])
    ),
    tag = "Pockets"
)]
pub async fn update_pocket(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdatePocketRequest>,
) -> Result<Json<ApiResponse<PocketResponse>>, AppError> {
    let result = state.pocket_service
        .update_pocket(id, auth_user.id, payload.name, payload.pocket_type)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(ApiResponse { data: PocketResponse::from(result) }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/pockets/{id}",
    params(
        ("id" = Uuid, Path, description = "Pocket ID")
    ),
    responses(
        (status = 204, description = "Pocket deleted successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(
        ("BearerAuth" = [])
    ),
    tag = "Pockets"
)]
pub async fn delete_pocket(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.pocket_service
        .delete_pocket(id, auth_user.id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(StatusCode::NO_CONTENT)
}

use axum::routing::get;
use axum::Router;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_pockets).post(create_pocket))
        .route("/:id", get(get_pocket).put(update_pocket).delete(delete_pocket))
}
