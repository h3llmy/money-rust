use crate::domain::auth::entity::User;
use crate::domain::auth::AuthRepository;
use crate::infrastructure::database::DbPool;
use crate::infrastructure::repository::UserDb;
use crate::infrastructure::schema::users;
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;
use chrono::Utc;

pub struct DieselAuthRepository {
    pool: DbPool,
}

impl DieselAuthRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuthRepository for DieselAuthRepository {
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, String> {
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;
        let result = users::table
            .filter(users::email.eq(email))
            .first::<UserDb>(&mut conn)
            .await
            .optional()
            .map_err(|e| e.to_string())?;
        
        Ok(result.map(map_db_to_domain))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, String> {
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;
        let result = users::table
            .filter(users::id.eq(id))
            .first::<UserDb>(&mut conn)
            .await
            .optional()
            .map_err(|e| e.to_string())?;
        
        Ok(result.map(map_db_to_domain))
    }

    async fn create(&self, email: &str, username: &str, password_hash: &str) -> Result<User, String> {
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;
        let db_model = UserDb {
            id: Uuid::new_v4(),
            email: email.to_string(),
            password_hash: password_hash.to_string(),
            username: username.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        diesel::insert_into(users::table)
            .values(&db_model)
            .execute(&mut conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(map_db_to_domain(db_model))
    }
}

fn map_db_to_domain(db: UserDb) -> User {
    User {
        id: db.id,
        email: db.email,
        password_hash: db.password_hash,
        username: db.username,
        created_at: db.created_at,
        updated_at: db.updated_at,
    }
}

