use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::domain::categories::entity::Category;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema, Clone)]
pub struct CategoryResponse {
    pub id: Uuid,
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
}

impl From<Category> for CategoryResponse {
    fn from(c: Category) -> Self {
        Self {
            id: c.id,
            name: c.name,
            type_: c.type_,
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct CreateCategoryRequest {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateCategoryRequest {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
}
