use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use futures_util::stream::BoxStream;
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

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct AiTransactionQuery {
    pub pocket_name: Option<String>,
    pub category_name: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub transaction_type: Option<String>,
    pub limit: Option<u32>,
}

#[async_trait]
pub trait AiClient: Send + Sync {
    async fn parse_notification(
        &self,
        title: &str,
        body: &str,
    ) -> Result<Option<AiParsedNotification>, String>;

    async fn parse_transaction_query(
        &self,
        query: &str,
        current_date: &str,
    ) -> Result<AiTransactionQuery, String>;

    async fn analyze_transactions(
        &self,
        transactions_json: &str,
        user_query: Option<&str>,
    ) -> Result<BoxStream<'static, Result<String, String>>, String>;
}
