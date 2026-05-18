use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::domain::pockets::dto::PocketResponse;
use crate::domain::categories::dto::CategoryResponse;
use crate::domain::transactions::dto::TransactionResponse;
use crate::domain::notifications::dto::NotificationResponse;

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[aliases(
    PocketApiResponse = ApiResponse<PocketResponse>,
    CategoryApiResponse = ApiResponse<CategoryResponse>,
    TransactionApiResponse = ApiResponse<TransactionResponse>,
    NotificationApiResponse = ApiResponse<NotificationResponse>,
    StringApiResponse = ApiResponse<String>,
)]
pub struct ApiResponse<T> {
    pub data: T,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[aliases(
    PocketPaginationResponse = PaginationResponse<Vec<PocketResponse>>,
    CategoryPaginationResponse = PaginationResponse<Vec<CategoryResponse>>,
    TransactionPaginationResponse = PaginationResponse<Vec<TransactionResponse>>,
    NotificationPaginationResponse = PaginationResponse<Vec<NotificationResponse>>,
)]
pub struct PaginationResponse<T> {
    pub data: T,
    pub page: u32,
    pub limit: u32,
    pub total_data: u64,
    pub total_page: u64,
}

impl<T> PaginationResponse<T> {
    pub fn new(data: T, page: u32, limit: u32, total_data: u64) -> Self {
        let total_page = (total_data as f64 / limit as f64).ceil() as u64;
        Self {
            data,
            page,
            limit,
            total_data,
            total_page,
        }
    }
}
