use crate::domain::categories::entity::Category;
use crate::domain::categories::CategoryRepository;
use crate::infrastructure::database::DbPool;
use crate::infrastructure::repository::CategoryDb;
use crate::infrastructure::schema::categories;
use crate::shared::pagination::PaginationQuery;
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

pub struct DieselCategoryRepository {
    pool: DbPool,
}

impl DieselCategoryRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CategoryRepository for DieselCategoryRepository {
    async fn find_all(&self, user_id: Uuid, pagination: PaginationQuery) -> Result<(Vec<Category>, u64), String> {
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;
        
        let mut count_query = categories::table.filter(categories::user_id.is_null().or(categories::user_id.eq(user_id))).into_boxed();
        let mut list_query = categories::table.filter(categories::user_id.is_null().or(categories::user_id.eq(user_id))).into_boxed();
        
        if let Some(search) = pagination.get_search() {
            let pattern = format!("%{}%", search);
            count_query = count_query.filter(categories::name.ilike(pattern.clone()));
            list_query = list_query.filter(categories::name.ilike(pattern));
        }

        let total_data = count_query
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| e.to_string())? as u64;

        let results = list_query
            .limit(pagination.get_limit() as i64)
            .offset(pagination.get_offset())
            .load::<CategoryDb>(&mut conn)
            .await
            .map_err(|e| e.to_string())?;
        
        Ok((results.into_iter().map(map_db_to_domain).collect(), total_data))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Category>, String> {
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;
        let result = categories::table
            .filter(categories::id.eq(id))
            .first::<CategoryDb>(&mut conn)
            .await
            .optional()
            .map_err(|e| e.to_string())?;
        
        Ok(result.map(map_db_to_domain))
    }

    async fn save(&self, category: Category) -> Result<Category, String> {
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;
        let db_model = CategoryDb {
            id: category.id,
            user_id: category.user_id,
            name: category.name,
            type_: category.type_,
        };

        diesel::insert_into(categories::table)
            .values(&db_model)
            .on_conflict(categories::id)
            .do_update()
            .set(&db_model)
            .execute(&mut conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(map_db_to_domain(db_model))
    }

    async fn delete(&self, id: Uuid) -> Result<(), String> {
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;
        diesel::delete(categories::table.filter(categories::id.eq(id)))
            .execute(&mut conn)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn map_db_to_domain(db: CategoryDb) -> Category {
    Category {
        id: db.id,
        user_id: db.user_id,
        name: db.name,
        type_: db.type_,
    }
}
