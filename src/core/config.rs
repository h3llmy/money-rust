pub struct Config {
    pub database_url: String,
    pub server_port: u16,
    pub log_level: String,
    pub environment: AppEnvironment,
    pub ollama_host: String,
    pub ollama_model: String,
    pub ai_provider: String,
    pub gemini_api_key: String,
    pub gemini_model: String,
    pub jwt_secret: String,
}

pub enum AppEnvironment {
    Development,
    Production,
}

impl Config {
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");

        let server_port = std::env::var("PORT")
            .unwrap_or_else(|_| "5008".to_string())
            .parse::<u16>()
            .expect("PORT must be a valid number");

        let log_level = std::env::var("RUST_LOG")
            .unwrap_or_else(|_| "mobile_money_backend=debug,tower_http=debug".to_string());

        let environment = match std::env::var("APP_ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string())
            .to_lowercase()
            .as_str()
        {
            "production" => AppEnvironment::Production,
            _ => AppEnvironment::Development,
        };

        let ollama_host = std::env::var("OLLAMA_HOST")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());
        
        let ollama_model = std::env::var("OLLAMA_MODEL")
            .unwrap_or_else(|_| "llama3:8b".to_string());
            
        let ai_provider = std::env::var("AI_PROVIDER")
            .unwrap_or_else(|_| "ollama".to_string());
            
        let gemini_api_key = std::env::var("GEMINI_API_KEY")
            .unwrap_or_else(|_| "".to_string());
            
        let gemini_model = std::env::var("GEMINI_MODEL")
            .unwrap_or_else(|_| "gemini-1.5-flash".to_string());

        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "super_secret_mobile_money_key_123456789".to_string());

        Self {
            database_url,
            server_port,
            log_level,
            environment,
            ollama_host,
            ollama_model,
            ai_provider,
            gemini_api_key,
            gemini_model,
            jwt_secret,
        }
    }
}
