use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use crate::shared::app_state::AppState;
use crate::core::jwt::decode_token;
use std::sync::Arc;
use uuid::Uuid;

pub struct AuthUser {
    pub id: Uuid,
    #[allow(dead_code)]
    pub email: String,
}

use axum::extract::FromRef;

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    Arc<AppState>: FromRef<S>,
{
    type Rejection = crate::shared::error::AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // 1. Get State reference
        let state = Arc::<AppState>::from_ref(state);

        // 2. Extract Authorization header
        let auth_header = parts.headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| crate::shared::error::AppError {
                status: StatusCode::UNAUTHORIZED,
                message: "Missing authorization header".to_string(),
            })?;

        if !auth_header.starts_with("Bearer ") {
            return Err(crate::shared::error::AppError {
                status: StatusCode::UNAUTHORIZED,
                message: "Invalid authorization header format".to_string(),
            });
        }

        let token = &auth_header[7..];

        // 3. Decode and validate the token
        let claims = decode_token(token, &state.jwt_secret, "access")
            .map_err(|e| crate::shared::error::AppError {
                status: StatusCode::UNAUTHORIZED,
                message: e,
            })?;

        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|e| crate::shared::error::AppError {
                status: StatusCode::UNAUTHORIZED,
                message: format!("Invalid user ID in token: {}", e),
            })?;

        Ok(AuthUser {
            id: user_id,
            email: claims.email,
        })
    }
}

