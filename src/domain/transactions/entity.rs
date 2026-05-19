use uuid::Uuid;
use chrono::{DateTime, Utc};
use bigdecimal::BigDecimal;

pub struct Transaction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub pocket_id: Uuid,
    pub category_id: Option<Uuid>,
    pub amount: BigDecimal,
    pub type_: String, // income, expense, or transfer
    pub title: String,
    pub transaction_time: DateTime<Utc>,
    pub destination_pocket_id: Option<Uuid>,
    pub description: Option<String>,
    pub status: String, // pending, resolved, rejected
}
