use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::domain::pockets::entity::Pocket;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema, Clone)]
pub struct PocketResponse {
    pub id: Uuid,
    pub name: String,
    pub pocket_type: String,
    pub balance: String,
    pub currency: String,
}

impl From<Pocket> for PocketResponse {
    fn from(p: Pocket) -> Self {
        Self {
            id: p.id,
            name: p.name,
            pocket_type: p._pocket_type,
            balance: p.balance.to_string(),
            currency: p.currency,
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct CreatePocketRequest {
    pub name: String,
    pub pocket_type: String,
    pub currency: String,
    pub balance: String,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdatePocketRequest {
    pub name: String,
    pub pocket_type: String,
}
