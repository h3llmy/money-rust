use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use bigdecimal::BigDecimal;
use crate::infrastructure::schema::*;

pub mod pocket_repository_impl;
pub mod notification_repository_impl;
pub mod category_repository_impl;
pub mod transaction_repository_impl;
pub mod auth_repository_impl;

// Pocket DB Model
#[derive(Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize, Debug)]
#[diesel(table_name = pockets)]
pub struct PocketDb {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub pocket_type: String,
    pub currency: String,
    pub balance: BigDecimal,
    pub updated_at: DateTime<Utc>,
}

// Category DB Model
#[derive(Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize, Debug)]
#[diesel(table_name = categories)]
pub struct CategoryDb {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub name: String,
    pub type_: String,
}

// Transaction DB Model
#[derive(Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize, Debug)]
#[diesel(table_name = transactions)]
pub struct TransactionDb {
    pub id: Uuid,
    pub user_id: Uuid,
    pub pocket_id: Uuid,
    pub category_id: Option<Uuid>,
    pub amount: BigDecimal,
    pub type_: String,
    pub title: String,
    pub transaction_time: DateTime<Utc>,
    pub destination_pocket_id: Option<Uuid>,
    pub description: Option<String>,
    pub status: String,
}

// User DB Model
#[derive(Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize, Debug)]
#[diesel(table_name = users)]
pub struct UserDb {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}


