use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;

pub mod ollama_client;
pub mod gemini_client;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AiParsedNotification {
    pub is_transaction: bool,
    pub amount: Option<BigDecimal>,
    pub transaction_type: Option<String>, // income, expense, transfer
    pub title: Option<String>,
    pub pocket: Option<String>,
    pub category: Option<String>,
    pub destination_pocket: Option<String>,
}

#[async_trait]
pub trait AiClient: Send + Sync {
    async fn parse_notification(
        &self,
        title: &str,
        body: &str,
    ) -> Result<Option<AiParsedNotification>, String>;

    async fn analyze_transactions(
        &self,
        transactions_json: &str,
        user_query: Option<&str>,
    ) -> Result<String, String>;
}
