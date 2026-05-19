use crate::domain::pockets::entity::Pocket;
use crate::domain::pockets::PocketRepository;
use crate::infrastructure::database::DbPool;
use crate::infrastructure::repository::PocketDb;
use crate::infrastructure::schema::pockets;
use crate::shared::pagination::PaginationQuery;
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;
use bigdecimal::BigDecimal;
use chrono::Utc;

pub struct DieselPocketRepository {
    pool: DbPool,
}

impl DieselPocketRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PocketRepository for DieselPocketRepository {
    async fn find_all(&self, user_id: Uuid, pagination: PaginationQuery) -> Result<(Vec<Pocket>, u64), String> {
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;
        
        let mut count_query = pockets::table.filter(pockets::user_id.eq(user_id)).into_boxed();
        let mut list_query = pockets::table.filter(pockets::user_id.eq(user_id)).into_boxed();
        
        if let Some(search) = pagination.get_search() {
            let pattern = format!("%{}%", search);
            count_query = count_query.filter(pockets::name.ilike(pattern.clone()));
            list_query = list_query.filter(pockets::name.ilike(pattern));
        }

        let total_data = count_query
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| e.to_string())? as u64;

        let results = list_query
            .limit(pagination.get_limit() as i64)
            .offset(pagination.get_offset())
            .load::<PocketDb>(&mut conn)
            .await
            .map_err(|e| e.to_string())?;
        
        Ok((results.into_iter().map(map_db_to_domain).collect(), total_data))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Pocket>, String> {
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;
        let result = pockets::table
            .filter(pockets::id.eq(id))
            .first::<PocketDb>(&mut conn)
            .await
            .optional()
            .map_err(|e| e.to_string())?;
        
        Ok(result.map(map_db_to_domain))
    }

    async fn save(&self, pocket: Pocket) -> Result<Pocket, String> {
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;
        let db_model = PocketDb {
            id: pocket.id,
            user_id: pocket.user_id,
            name: pocket.name,
            pocket_type: pocket._pocket_type,
            currency: pocket.currency,
            balance: pocket.balance,
            updated_at: Utc::now(),
        };

        diesel::insert_into(pockets::table)
            .values(&db_model)
            .on_conflict(pockets::id)
            .do_update()
            .set(&db_model)
            .execute(&mut conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(map_db_to_domain(db_model))
    }

    async fn update_balance(&self, id: Uuid, amount: BigDecimal) -> Result<(), String> {
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;
        
        diesel::update(pockets::table.filter(pockets::id.eq(id)))
            .set((
                pockets::balance.eq(pockets::balance + amount),
                pockets::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| e.to_string())?;
            
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), String> {
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;
        
        diesel::delete(pockets::table.filter(pockets::id.eq(id)))
            .execute(&mut conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}

fn map_db_to_domain(db: PocketDb) -> Pocket {
    Pocket {
        id: db.id,
        user_id: db.user_id,
        name: db.name,
        _pocket_type: db.pocket_type,
        currency: db.currency,
        balance: db.balance,
        _updated_at: db.updated_at,
    }
}
