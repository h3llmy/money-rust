pub mod entity;
pub mod service;
pub mod dto;

use crate::domain::pockets::entity::Pocket;
use crate::shared::pagination::PaginationQuery;
use async_trait::async_trait;
use uuid::Uuid;
use bigdecimal::BigDecimal;

#[async_trait]
pub trait PocketRepository: Send + Sync {
    async fn find_all(&self, user_id: Uuid, pagination: PaginationQuery) -> Result<(Vec<Pocket>, u64), String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Pocket>, String>;
    async fn save(&self, pocket: Pocket) -> Result<Pocket, String>;
    async fn update_balance(&self, id: Uuid, amount: BigDecimal) -> Result<(), String>;
    async fn delete(&self, id: Uuid) -> Result<(), String>;
}
