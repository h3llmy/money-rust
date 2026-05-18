use axum::{extract::{State, Query}, Json, http::StatusCode};
use crate::shared::app_state::AppState;
use crate::domain::notifications::dto::{CreateNotificationRequest, NotificationResponse};
use crate::domain::notifications::entity::{NotificationInbox, NotificationStatus};
use crate::shared::pagination::PaginationQuery;
use crate::shared::response::PaginationResponse;
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

#[utoipa::path(
    get,
    path = "/api/v1/inbox",
    params(PaginationQuery),
    responses(
        (status = 200, description = "List notifications", body = NotificationPaginationResponse)
    ),
    tag = "Inbox"
)]
pub async fn list_unresolved(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginationResponse<Vec<NotificationResponse>>>, (StatusCode, String)> {
    let (data, total) = state.notification_service
        .list_unresolved(pagination.clone())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let response = PaginationResponse::new(
        data.into_iter().map(NotificationResponse::from).collect(),
        pagination.get_page(),
        pagination.get_limit(),
        total,
    );

    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/api/v1/inbox/sync",
    request_body = Vec<CreateNotificationRequest>,
    responses(
        (status = 202, description = "Notifications synced successfully")
    ),
    tag = "Inbox"
)]
pub async fn sync_inbox(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Vec<CreateNotificationRequest>>,
) -> Result<StatusCode, (StatusCode, String)> {
    tracing::info!("Syncing inbox: {:?}", payload);
    let notifications = payload.into_iter().map(|p| NotificationInbox {
        id: Uuid::new_v4(),
        app_package: p.app_package,
        raw_title: p.raw_title,
        raw_body: p.raw_body,
        received_at: Utc::now(),
        status: NotificationStatus::Pending,
        transaction_id: None,
        amount: None,
        type_: None,
        pocket_id: None,
        category_id: None,
        destination_pocket_id: None,
        title: None,
    }).collect();

    state.notification_service
        .ingest_sync(notifications)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(StatusCode::ACCEPTED)
}

use axum::routing::{get, post};
use axum::Router;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/sync", post(sync_inbox))
        .route("/", get(list_unresolved))
}
