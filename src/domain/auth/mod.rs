pub mod entity;
pub mod service;
pub mod dto;

use crate::domain::auth::entity::User;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait AuthRepository: Send + Sync {
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, String>;
    async fn create(&self, email: &str, username: &str, password_hash: &str) -> Result<User, String>;
}
