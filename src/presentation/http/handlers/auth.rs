use axum::{extract::State, Json, http::StatusCode};
use crate::shared::app_state::AppState;
use crate::shared::auth::AuthUser;
use crate::domain::auth::dto::{RegisterRequest, LoginRequest, AuthResponse, UserResponse};
use std::sync::Arc;
use validator::Validate;

#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "User registered successfully", body = AuthResponse),
        (status = 400, description = "Invalid input or email already exists", body = String),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "Auth"
)]
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    payload.validate().map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let res = state.auth_service
        .register(payload)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    Ok(Json(res))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "User logged in successfully", body = AuthResponse),
        (status = 400, description = "Invalid credentials", body = String),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "Auth"
)]
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    payload.validate().map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let res = state.auth_service
        .login(payload)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    Ok(Json(res))
}

#[utoipa::path(
    get,
    path = "/api/v1/users/me",
    responses(
        (status = 200, description = "Get current user profile", body = UserResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "Users",
    security(
        ("BearerAuth" = [])
    )
)]
pub async fn get_me(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
) -> Result<Json<UserResponse>, (StatusCode, String)> {
    let user = state.auth_service
        .get_user_by_id(auth_user.id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "User not found".to_string()))?;

    Ok(Json(UserResponse::from(user)))
}

use axum::routing::post;
use axum::routing::get;
use axum::Router;

pub fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
}

pub fn user_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/me", get(get_me))
}

