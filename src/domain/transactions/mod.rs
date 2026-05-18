pub mod entity;
pub mod service;
pub mod dto;

use crate::domain::transactions::entity::Transaction;
use crate::shared::pagination::PaginationQuery;
use async_trait::async_trait;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[async_trait]
pub trait TransactionRepository: Send + Sync {
    async fn find_all(
        &self, 
        pocket_id: Option<Uuid>, 
        start_date: Option<DateTime<Utc>>, 
        end_date: Option<DateTime<Utc>>, 
        type_: Option<String>,
        pagination: PaginationQuery,
    ) -> Result<(Vec<Transaction>, u64), String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Transaction>, String>;
    async fn save(&self, transaction: Transaction) -> Result<Transaction, String>;
    async fn delete(&self, id: Uuid) -> Result<(), String>;
}
