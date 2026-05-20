use axum::{extract::{State, Path, Query}, Json, http::StatusCode};
use crate::shared::app_state::AppState;
use crate::shared::auth::AuthUser;
use crate::domain::transactions::dto::{TransactionResponse, CreateTransactionRequest, UpdateTransactionRequest, TransactionFilter, ResolveTransactionRequest};
use crate::shared::response::{ApiResponse, PaginationResponse};
use crate::shared::error::AppError;
use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;
use crate::domain::pockets::dto::PocketResponse;
use crate::domain::categories::dto::CategoryResponse;

use crate::shared::pagination::PaginationQuery;

#[utoipa::path(
    get,
    path = "/api/v1/transactions",
    params(TransactionFilter),
    responses(
        (status = 200, description = "List all transactions", body = TransactionPaginationResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(
        ("BearerAuth" = [])
    ),
    tag = "Transactions"
)]
pub async fn list_transactions(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Query(filter): Query<TransactionFilter>,
) -> Result<Json<PaginationResponse<Vec<TransactionResponse>>>, AppError> {
    let pagination = PaginationQuery {
        page: filter.page,
        limit: filter.limit,
        search: filter.search.clone(),
        sort: filter.sort.clone(),
        sort_order: filter.sort_order.clone(),
    };

    let (data, total) = state.transaction_service
        .list_transactions(
            auth_user.id,
            filter.pocket_id, 
            filter.start_date, 
            filter.end_date, 
            filter.type_, 
            pagination.clone()
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Batch fetch pockets and categories to populate relations
    let (pockets_list, _) = state.pocket_service
        .list_all_pockets(auth_user.id, PaginationQuery { page: Some(1), limit: Some(1000), ..Default::default() })
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let (categories_list, _) = state.category_service
        .list_categories(auth_user.id, PaginationQuery { page: Some(1), limit: Some(1000), ..Default::default() })
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let pockets_map: HashMap<Uuid, PocketResponse> = pockets_list
        .into_iter()
        .map(|p| (p.id, PocketResponse::from(p)))
        .collect();

    let categories_map: HashMap<Uuid, CategoryResponse> = categories_list
        .into_iter()
        .map(|c| (c.id, CategoryResponse::from(c)))
        .collect();

    let mut populated_data = Vec::new();
    for t in data {
        let pocket = pockets_map.get(&t.pocket_id).cloned();
        let category = t.category_id.and_then(|cid| categories_map.get(&cid).cloned());
        let destination_pocket = t.destination_pocket_id.and_then(|dpid| pockets_map.get(&dpid).cloned());

        populated_data.push(TransactionResponse {
            id: t.id,
            pocket_id: t.pocket_id,
            pocket,
            category_id: t.category_id,
            category,
            amount: t.amount.to_string(),
            type_: t.type_,
            title: t.title,
            transaction_time: t.transaction_time,
            destination_pocket_id: t.destination_pocket_id,
            destination_pocket,
            description: t.description,
            status: t.status,
        });
    }

    let response = PaginationResponse::new(
        populated_data,
        pagination.get_page(),
        pagination.get_limit(),
        total,
    );

    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/api/v1/transactions",
    request_body = CreateTransactionRequest,
    responses(
        (status = 200, description = "Create a new transaction", body = TransactionApiResponse),
        (status = 400, description = "Bad Request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(
        ("BearerAuth" = [])
    ),
    tag = "Transactions"
)]
pub async fn create_transaction(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Json(payload): Json<CreateTransactionRequest>,
) -> Result<Json<ApiResponse<TransactionResponse>>, AppError> {
    use std::str::FromStr;
    let amount = bigdecimal::BigDecimal::from_str(&payload.amount)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid amount: {}", e)))?;

    let result = state.transaction_service
        .create_transaction(
            auth_user.id,
            payload.pocket_id,
            payload.category_id,
            amount,
            payload.type_,
            payload.title,
            payload.transaction_time,
            payload.destination_pocket_id,
            payload.description,
            payload.status.unwrap_or_else(|| "resolved".to_string()),
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(ApiResponse { data: TransactionResponse::from(result) }))
}

#[utoipa::path(
    put,
    path = "/api/v1/transactions/{id}",
    params(
        ("id" = Uuid, Path, description = "Transaction ID")
    ),
    request_body = UpdateTransactionRequest,
    responses(
        (status = 200, description = "Update a transaction", body = TransactionApiResponse),
        (status = 400, description = "Bad Request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(
        ("BearerAuth" = [])
    ),
    tag = "Transactions"
)]
pub async fn update_transaction(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTransactionRequest>,
) -> Result<Json<ApiResponse<TransactionResponse>>, AppError> {
    use std::str::FromStr;
    let amount = bigdecimal::BigDecimal::from_str(&payload.amount)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid amount: {}", e)))?;

    let result = state.transaction_service
        .update_transaction(
            id,
            auth_user.id,
            payload.pocket_id,
            payload.category_id,
            amount,
            payload.type_,
            payload.title,
            payload.transaction_time,
            payload.destination_pocket_id,
            payload.description,
            payload.status,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(ApiResponse { data: TransactionResponse::from(result) }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/transactions/{id}",
    params(
        ("id" = Uuid, Path, description = "Transaction ID")
    ),
    responses(
        (status = 204, description = "Transaction voided successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(
        ("BearerAuth" = [])
    ),
    tag = "Transactions"
)]
pub async fn void_transaction(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.transaction_service
        .void_transaction(id, auth_user.id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/transactions/{id}",
    params(
        ("id" = Uuid, Path, description = "Transaction ID")
    ),
    responses(
        (status = 200, description = "Get transaction details", body = TransactionApiResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Transaction not found", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(
        ("BearerAuth" = [])
    ),
    tag = "Transactions"
)]
pub async fn get_transaction(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<TransactionResponse>>, AppError> {
    let t = state.transaction_service
        .get_transaction_by_id(id, auth_user.id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Transaction not found".to_string()))?;

    let pocket = state.pocket_service
        .get_pocket_by_id(t.pocket_id, auth_user.id)
        .await
        .ok()
        .flatten()
        .map(PocketResponse::from);

    let category = if let Some(cid) = t.category_id {
        state.category_service
            .get_category_by_id(cid, auth_user.id)
            .await
            .ok()
            .flatten()
            .map(CategoryResponse::from)
    } else {
        None
    };

    let destination_pocket = if let Some(dpid) = t.destination_pocket_id {
        state.pocket_service
            .get_pocket_by_id(dpid, auth_user.id)
            .await
            .ok()
            .flatten()
            .map(PocketResponse::from)
    } else {
        None
    };

    let mut response = TransactionResponse::from(t);
    response.pocket = pocket;
    response.category = category;
    response.destination_pocket = destination_pocket;

    Ok(Json(ApiResponse { data: response }))
}

#[utoipa::path(
    post,
    path = "/api/v1/transactions/{id}/resolve",
    params(
        ("id" = Uuid, Path, description = "Transaction ID")
    ),
    request_body = ResolveTransactionRequest,
    responses(
        (status = 200, description = "Resolve a pending transaction", body = TransactionApiResponse),
        (status = 400, description = "Bad Request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Transaction not found", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(
        ("BearerAuth" = [])
    ),
    tag = "Transactions"
)]
pub async fn resolve_transaction(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<ResolveTransactionRequest>,
) -> Result<Json<ApiResponse<TransactionResponse>>, AppError> {
    use std::str::FromStr;
    
    let amount = if let Some(amt_str) = payload.amount.as_ref() {
        let parsed = bigdecimal::BigDecimal::from_str(amt_str)
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid amount: {}", e)))?;
        Some(parsed)
    } else {
        None
    };

    let result = state.transaction_service
        .resolve_transaction(
            id,
            auth_user.id,
            payload.pocket_id,
            payload.category_id,
            amount,
            payload.type_,
            payload.title,
            payload.destination_pocket_id,
            payload.description,
        )
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    Ok(Json(ApiResponse { data: TransactionResponse::from(result) }))
}

#[utoipa::path(
    post,
    path = "/api/v1/transactions/{id}/reject",
    params(
        ("id" = Uuid, Path, description = "Transaction ID")
    ),
    responses(
        (status = 200, description = "Reject a pending transaction", body = TransactionApiResponse),
        (status = 400, description = "Bad Request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Transaction not found", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(
        ("BearerAuth" = [])
    ),
    tag = "Transactions"
)]
pub async fn reject_transaction(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<TransactionResponse>>, AppError> {
    let result = state.transaction_service
        .reject_transaction(id, auth_user.id)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    Ok(Json(ApiResponse { data: TransactionResponse::from(result) }))
}

use crate::domain::transactions::dto::{AiAnalyzeRequest, AiAnalyzeResponse};

#[utoipa::path(
    post,
    path = "/api/v1/transactions/ai-analyze",
    request_body = AiAnalyzeRequest,
    responses(
        (status = 200, description = "Perform AI transaction analysis", body = AiAnalyzeResponse),
        (status = 400, description = "Bad Request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 502, description = "Bad Gateway", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(
        ("BearerAuth" = [])
    ),
    tag = "Transactions"
)]
pub async fn ai_analyze(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Json(payload): Json<AiAnalyzeRequest>,
) -> Result<Json<AiAnalyzeResponse>, AppError> {
    let pagination = PaginationQuery {
        page: Some(1),
        limit: Some(200), // Get up to 200 transactions
        search: None,
        sort: None,
        sort_order: None,
    };

    let (data, _) = state.transaction_service
        .list_transactions(
            auth_user.id,
            None,
            None,
            None,
            None,
            pagination
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let response_data = data.into_iter()
        .map(TransactionResponse::from)
        .collect::<Vec<_>>();

    let transactions_json = serde_json::to_string_pretty(&response_data)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let analysis = state.ai_client
        .analyze_transactions(&transactions_json, payload.query.as_deref())
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e))?;

    Ok(Json(AiAnalyzeResponse { analysis }))
}

use axum::routing::get;
use axum::Router;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_transactions).post(create_transaction))
        .route("/ai-analyze", axum::routing::post(ai_analyze))
        .route("/:id", get(get_transaction).delete(void_transaction).put(update_transaction))
        .route("/:id/resolve", get(resolve_transaction).post(resolve_transaction))
        .route("/:id/reject", get(reject_transaction).post(reject_transaction))
}
