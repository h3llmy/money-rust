pub mod entity;
pub mod service;
pub mod dto;

use crate::domain::notifications::entity::NotificationInbox;
use crate::shared::pagination::PaginationQuery;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait NotificationRepository: Send + Sync {
    async fn find_unresolved(&self, user_id: Uuid, pagination: PaginationQuery) -> Result<(Vec<NotificationInbox>, u64), String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<NotificationInbox>, String>;
    async fn save(&self, notification: NotificationInbox) -> Result<NotificationInbox, String>;
    async fn update_status(&self, id: Uuid, status: String, transaction_id: Option<Uuid>) -> Result<(), String>;
}
