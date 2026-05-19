use uuid::Uuid;
use chrono::{DateTime, Utc};
use bigdecimal::BigDecimal;

pub struct Pocket {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub _pocket_type: String,
    pub currency: String,
    pub balance: BigDecimal,
    pub _updated_at: DateTime<Utc>,
}
