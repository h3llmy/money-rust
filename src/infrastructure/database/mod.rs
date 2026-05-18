pub mod seeder;

use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::bb8::Pool;
use diesel_async::AsyncPgConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use diesel::pg::PgConnection;
use diesel::Connection;

pub type DbPool = Pool<AsyncPgConnection>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

pub async fn get_connection_pool(database_url: &str) -> DbPool {
    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    Pool::builder()
        .build(config)
        .await
        .expect("Failed to create pool.")
}

pub fn run_migrations(database_url: &str) {
    let mut conn = PgConnection::establish(database_url)
        .expect("Failed to connect to database for migrations");
    
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");
    
    tracing::info!("Database migrations completed successfully");
}
