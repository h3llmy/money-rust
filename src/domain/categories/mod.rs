pub mod entity;
pub mod service;
pub mod dto;

use crate::domain::categories::entity::Category;
use crate::shared::pagination::PaginationQuery;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait CategoryRepository: Send + Sync {
    async fn find_all(&self, pagination: PaginationQuery) -> Result<(Vec<Category>, u64), String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Category>, String>;
    async fn save(&self, category: Category) -> Result<Category, String>;
    async fn delete(&self, id: Uuid) -> Result<(), String>;
}
