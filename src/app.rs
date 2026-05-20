use std::sync::Arc;
use crate::core::config::Config;
use crate::infrastructure::database::{get_connection_pool, run_migrations};
use crate::infrastructure::repository::{
    pocket_repository_impl::DieselPocketRepository,
    notification_repository_impl::InMemoryNotificationRepository,
    category_repository_impl::DieselCategoryRepository,
    transaction_repository_impl::DieselTransactionRepository,
    auth_repository_impl::DieselAuthRepository,
};
use crate::infrastructure::ai::ollama_client::OllamaClient;
use crate::infrastructure::ai::gemini_client::GeminiClient;
use crate::domain::pockets::service::PocketService;
use crate::domain::notifications::service::NotificationService;
use crate::domain::categories::service::CategoryService;
use crate::domain::transactions::service::TransactionService;
use crate::domain::auth::service::AuthService;
use crate::shared::app_state::AppState;
use crate::presentation::http::create_router;

pub async fn run(config: Config) {
    let config = Arc::new(config);
    
    // 1. Run migrations synchronously before anything else
    run_migrations(&config.database_url);

    // 2. Initialize async connection pool
    let pool = get_connection_pool(&config.database_url).await;

    // 3. Seed default data
    if let Err(e) = crate::infrastructure::database::seeder::seed_data(&pool).await {
        tracing::error!("Failed to seed database: {}", e);
    }

    // Infrastructure
    let ai_client: Arc<dyn crate::infrastructure::ai::AiClient> = match config.ai_provider.to_lowercase().as_str() {
        "gemini" => Arc::new(GeminiClient::new(config.clone())),
        _ => Arc::new(OllamaClient::new(config.clone())),
    };

    // Repositories
    let pocket_repo = Arc::new(DieselPocketRepository::new(pool.clone()));
    let notification_repo = Arc::new(InMemoryNotificationRepository::new());
    let category_repo = Arc::new(DieselCategoryRepository::new(pool.clone()));
    let transaction_repo = Arc::new(DieselTransactionRepository::new(pool.clone()));
    let auth_repo = Arc::new(DieselAuthRepository::new(pool.clone()));

    // Services
    let pocket_service = Arc::new(PocketService::new(pocket_repo.clone()));
    let category_service = Arc::new(CategoryService::new(category_repo.clone()));
    let transaction_service = Arc::new(TransactionService::new(transaction_repo.clone(), pocket_repo.clone()));
    let notification_service = Arc::new(NotificationService::new(
        notification_repo.clone(),
        pocket_repo.clone(),
        category_repo.clone(),
        transaction_service.clone(),
        ai_client.clone(),
    ));
    let auth_service = Arc::new(AuthService::new(auth_repo.clone(), config.jwt_secret.clone()));

    let state = Arc::new(AppState {
        pocket_service,
        notification_service,
        category_service,
        transaction_service,
        auth_service,
        ai_client: ai_client.clone(),
        jwt_secret: config.jwt_secret.clone(),
    });

    let app = create_router(state);

    let addr = format!("0.0.0.0:{}", config.server_port);
    tracing::info!("listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

