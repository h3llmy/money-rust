pub mod handlers;

use crate::shared::app_state::AppState;
use axum::{routing::get, Router};
use tower_http::trace::TraceLayer;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::domain::pockets::dto::*;
use crate::domain::categories::dto::*;
use crate::domain::transactions::dto::*;
use crate::domain::notifications::dto::*;
use crate::shared::response::*;
use crate::shared::pagination::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::pockets::list_pockets,
        handlers::pockets::create_pocket,
        handlers::pockets::get_pocket,
        handlers::pockets::update_pocket,
        handlers::pockets::delete_pocket,
        handlers::categories::list_categories,
        handlers::categories::create_category,
        handlers::categories::update_category,
        handlers::categories::delete_category,
        handlers::transactions::list_transactions,
        handlers::transactions::create_transaction,
        handlers::transactions::get_transaction,
        handlers::transactions::update_transaction,
        handlers::transactions::void_transaction,
        handlers::transactions::resolve_transaction,
        handlers::transactions::reject_transaction,
        handlers::notifications::list_unresolved,
        handlers::notifications::sync_inbox,
    ),
    components(
        schemas(
            PocketResponse, CreatePocketRequest, UpdatePocketRequest,
            CategoryResponse, CreateCategoryRequest, UpdateCategoryRequest,
            TransactionResponse, CreateTransactionRequest, UpdateTransactionRequest, ResolveTransactionRequest,
            CreateNotificationRequest, NotificationResponse,
            StringApiResponse, PocketApiResponse, CategoryApiResponse, TransactionApiResponse, NotificationApiResponse,
            PocketPaginationResponse, CategoryPaginationResponse, TransactionPaginationResponse, NotificationPaginationResponse,
            PaginationQuery, SortOrder
        )
    ),
    tags(
        (name = "Pockets", description = "Pocket management endpoints"),
        (name = "Categories", description = "Category management endpoints"),
        (name = "Transactions", description = "Transaction management endpoints"),
        (name = "Inbox", description = "Notification inbox endpoints")
    )
)]
struct ApiDoc;

pub fn create_router(state: Arc<AppState>) -> Router {
    let api_v1 = Router::new()
        .nest("/pockets", handlers::pockets::routes())
        .nest("/categories", handlers::categories::routes())
        .nest("/transactions", handlers::transactions::routes())
        .nest("/inbox", handlers::notifications::routes());

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/health", get(|| async { "OK" }))
        .nest("/api/v1", api_v1)
        .layer(TraceLayer::new_for_http()) // Add HTTP request/response logging
        .with_state(state)
}
