use crate::infrastructure::database::DbPool;
use crate::infrastructure::schema::{pockets, categories};
use crate::infrastructure::repository::{PocketDb, CategoryDb};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;
use chrono::Utc;
use bigdecimal::BigDecimal;

pub async fn seed_data(pool: &DbPool) -> Result<(), String> {
    let mut conn = pool.get().await.map_err(|e| e.to_string())?;

    // 1. Seed Pockets if empty
    let pocket_count: i64 = pockets::table.count().get_result(&mut conn).await.map_err(|e| e.to_string())?;
    
    if pocket_count == 0 {
        tracing::info!("Seeding default pockets...");
        let default_pockets = vec![
            PocketDb {
                id: Uuid::new_v4(),
                user_id: Uuid::nil(),
                name: "Main Wallet".to_string(),
                pocket_type: "cash".to_string(),
                currency: "IDR".to_string(),
                balance: BigDecimal::from(0),
                updated_at: Utc::now(),
            },
            PocketDb {
                id: Uuid::new_v4(),
                user_id: Uuid::nil(),
                name: "Bank Account".to_string(),
                pocket_type: "bank".to_string(),
                currency: "IDR".to_string(),
                balance: BigDecimal::from(0),
                updated_at: Utc::now(),
            },
        ];

        diesel::insert_into(pockets::table)
            .values(&default_pockets)
            .execute(&mut conn)
            .await
            .map_err(|e| e.to_string())?;
    }

    // 2. Seed Categories if empty
    let category_count: i64 = categories::table.count().get_result(&mut conn).await.map_err(|e| e.to_string())?;

    if category_count == 0 {
        tracing::info!("Seeding default categories...");
        let default_categories = vec![
            // Expenses
            CategoryDb { id: Uuid::new_v4(), user_id: None, name: "Food & Beverage".to_string(), type_: "expense".to_string() },
            CategoryDb { id: Uuid::new_v4(), user_id: None, name: "Transport".to_string(), type_: "expense".to_string() },
            CategoryDb { id: Uuid::new_v4(), user_id: None, name: "Shopping".to_string(), type_: "expense".to_string() },
            CategoryDb { id: Uuid::new_v4(), user_id: None, name: "Bills & Utilities".to_string(), type_: "expense".to_string() },
            // Income
            CategoryDb { id: Uuid::new_v4(), user_id: None, name: "Salary".to_string(), type_: "income".to_string() },
            CategoryDb { id: Uuid::new_v4(), user_id: None, name: "Bonus".to_string(), type_: "income".to_string() },
        ];

        diesel::insert_into(categories::table)
            .values(&default_categories)
            .execute(&mut conn)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}
