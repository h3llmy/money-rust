use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use crate::core::config::Config;

mod app;
mod core;
mod domain;
mod infrastructure;
mod presentation;
mod shared;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    
    // 1. Load Configuration
    let config = Config::from_env();

    // 2. Initialize Tracing using config
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| config.log_level.clone().into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 3. Start Application
    app::run(config).await;
}
