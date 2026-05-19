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
    pub email: String,
}

use axum::extract::FromRef;

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    Arc<AppState>: FromRef<S>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // 1. Get State reference
        let state = Arc::<AppState>::from_ref(state);

        // 2. Extract Authorization header
        let auth_header = parts.headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Missing authorization header".to_string()))?;

        if !auth_header.starts_with("Bearer ") {
            return Err((StatusCode::UNAUTHORIZED, "Invalid authorization header format".to_string()));
        }

        let token = &auth_header[7..];

        // 3. Decode and validate the token
        let claims = decode_token(token, &state.jwt_secret)
            .map_err(|e| (StatusCode::UNAUTHORIZED, e))?;

        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|e| (StatusCode::UNAUTHORIZED, format!("Invalid user ID in token: {}", e)))?;

        Ok(AuthUser {
            id: user_id,
            email: claims.email,
        })
    }
}

