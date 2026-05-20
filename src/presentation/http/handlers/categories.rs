use axum::{extract::{State, Path, Query}, Json, http::StatusCode};
use crate::shared::app_state::AppState;
use crate::shared::auth::AuthUser;
use crate::domain::categories::dto::{CategoryResponse, CreateCategoryRequest, UpdateCategoryRequest};
use crate::shared::pagination::PaginationQuery;
use crate::shared::response::{ApiResponse, PaginationResponse};
use crate::shared::error::AppError;
use std::sync::Arc;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/api/v1/categories",
    params(PaginationQuery),
    responses(
        (status = 200, description = "List all categories", body = CategoryPaginationResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(
        ("BearerAuth" = [])
    ),
    tag = "Categories"
)]
pub async fn list_categories(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginationResponse<Vec<CategoryResponse>>>, AppError> {
    let (data, total) = state.category_service
        .list_categories(auth_user.id, pagination.clone())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let response = PaginationResponse::new(
        data.into_iter().map(CategoryResponse::from).collect(),
        pagination.get_page(),
        pagination.get_limit(),
        total,
    );

    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/api/v1/categories",
    request_body = CreateCategoryRequest,
    responses(
        (status = 200, description = "Create a new category", body = CategoryApiResponse),
        (status = 400, description = "Bad Request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(
        ("BearerAuth" = [])
    ),
    tag = "Categories"
)]
pub async fn create_category(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Json(payload): Json<CreateCategoryRequest>,
) -> Result<Json<ApiResponse<CategoryResponse>>, AppError> {
    let result = state.category_service
        .create_category(auth_user.id, payload.name, payload.type_)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(ApiResponse { data: CategoryResponse::from(result) }))
}

#[utoipa::path(
    put,
    path = "/api/v1/categories/{id}",
    params(
        ("id" = Uuid, Path, description = "Category ID")
    ),
    request_body = UpdateCategoryRequest,
    responses(
        (status = 200, description = "Update a category", body = CategoryApiResponse),
        (status = 400, description = "Bad Request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(
        ("BearerAuth" = [])
    ),
    tag = "Categories"
)]
pub async fn update_category(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateCategoryRequest>,
) -> Result<Json<ApiResponse<CategoryResponse>>, AppError> {
    let result = state.category_service
        .update_category(id, auth_user.id, payload.name, payload.type_)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(ApiResponse { data: CategoryResponse::from(result) }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/categories/{id}",
    params(
        ("id" = Uuid, Path, description = "Category ID")
    ),
    responses(
        (status = 204, description = "Category deleted successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(
        ("BearerAuth" = [])
    ),
    tag = "Categories"
)]
pub async fn delete_category(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.category_service
        .delete_category(id, auth_user.id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(StatusCode::NO_CONTENT)
}

use axum::routing::{get, delete};
use axum::Router;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_categories).post(create_category))
        .route("/:id", delete(delete_category).put(update_category))
}
