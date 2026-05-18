use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::domain::transactions::entity::Transaction;
use utoipa::{ToSchema, IntoParams};

use crate::domain::pockets::dto::PocketResponse;
use crate::domain::categories::dto::CategoryResponse;

#[derive(Serialize, ToSchema, Clone)]
pub struct TransactionResponse {
    pub id: Uuid,
    pub pocket_id: Uuid,
    pub pocket: Option<PocketResponse>,
    pub category_id: Option<Uuid>,
    pub category: Option<CategoryResponse>,
    pub amount: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub title: String,
    pub transaction_time: DateTime<Utc>,
    pub destination_pocket_id: Option<Uuid>,
    pub destination_pocket: Option<PocketResponse>,
    pub description: Option<String>,
    pub status: String,
}

impl From<Transaction> for TransactionResponse {
    fn from(t: Transaction) -> Self {
        Self {
            id: t.id,
            pocket_id: t.pocket_id,
            pocket: None,
            category_id: t.category_id,
            category: None,
            amount: t.amount.to_string(),
            type_: t.type_,
            title: t.title,
            transaction_time: t.transaction_time,
            destination_pocket_id: t.destination_pocket_id,
            destination_pocket: None,
            description: t.description,
            status: t.status,
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct CreateTransactionRequest {
    pub pocket_id: Uuid,
    pub category_id: Option<Uuid>,
    pub amount: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub title: String,
    pub transaction_time: DateTime<Utc>,
    pub destination_pocket_id: Option<Uuid>,
    pub description: Option<String>,
    pub status: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateTransactionRequest {
    pub pocket_id: Uuid,
    pub category_id: Option<Uuid>,
    pub amount: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub title: String,
    pub transaction_time: DateTime<Utc>,
    pub destination_pocket_id: Option<Uuid>,
    pub description: Option<String>,
    pub status: Option<String>,
}

use crate::shared::pagination::SortOrder;

#[derive(Deserialize, IntoParams)]
pub struct TransactionFilter {
    pub pocket_id: Option<Uuid>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub search: Option<String>,
    pub sort: Option<String>,
    #[param(inline)]
    pub sort_order: Option<SortOrder>,
}

#[derive(Deserialize, ToSchema)]
pub struct ResolveTransactionRequest {
    pub pocket_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub amount: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub title: Option<String>,
    pub destination_pocket_id: Option<Uuid>,
    pub description: Option<String>,
}
