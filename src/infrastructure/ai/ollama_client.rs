use serde::{Deserialize, Serialize};
use crate::core::config::Config;
use std::sync::Arc;

#[derive(Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    tools: Vec<Tool>,
    stream: bool,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct Tool {
    #[serde(rename = "type")]
    tool_type: String, // "function"
    function: ToolFunction,
}

#[derive(Serialize)]
struct ToolFunction {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Deserialize)]
struct OllamaChatResponse {
    message: AssistantMessage,
}

#[derive(Deserialize)]
struct AssistantMessage {
    tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Deserialize)]
struct ToolCall {
    function: ToolCallFunction,
}

#[derive(Deserialize)]
struct ToolCallFunction {
    name: String,
    arguments: serde_json::Value,
}

use super::{AiClient, AiParsedNotification};
use async_trait::async_trait;

pub struct OllamaClient {
    config: Arc<Config>,
    client: reqwest::Client,
}

impl OllamaClient {
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl AiClient for OllamaClient {
    async fn parse_notification(
        &self,
        title: &str,
        body: &str,
    ) -> Result<Option<AiParsedNotification>, String> {
        let system_prompt = format!(
            "You are a financial assistant. Analyze notification messages to determine if they are financial transactions.\n\n\
            ### RULES:\n\
            1. Set 'is_transaction' to true if and only if the notification describes a completed or pending financial transaction (Transfer, Payment, Top-up, Income, Expense, or Bill).\n\
            2. If 'is_transaction' is true, you must extract: amount, transaction_type ('income', 'expense', or 'transfer'), title, pocket, and category.\n\
            3. If the message is not a financial transaction (e.g. vague, chat, OTP, verification, system alert, etc.), set 'is_transaction' to false.\n\n\
            ### EXAMPLES:\n\
            - Input: 'Transfer Rp 50.000 ke Tokopedia' -> is_transaction = true, amount = 50000, transaction_type = 'expense', title = 'Tokopedia'.\n\
            - Input: 'Anda menerima transfer sebesar Rp 100.000 dari Budi' -> is_transaction = true, amount = 100000, transaction_type = 'income', title = 'Transfer from Budi'.\n\
            - Input: 'Gunakan OTP 123456 untuk masuk ke akun Anda' -> is_transaction = false.\n\
            - Input: 'Uang' -> is_transaction = false.\n\n\
            Extract the likely pocket/wallet name and category from the text if possible.",
        );

        let tools = vec![Tool {
            tool_type: "function".to_string(),
            function: ToolFunction {
                name: "extract_transaction".to_string(),
                description: "Extract financial transaction details from a notification message".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "required": ["is_transaction"],
                    "properties": {
                        "is_transaction": {
                            "type": "boolean",
                            "description": "True if the notification is a financial transaction (income, expense, transfer), false otherwise"
                        },
                        "amount": {
                            "type": "number",
                            "description": "The transaction amount"
                        },
                        "transaction_type": {
                            "type": "string",
                            "enum": ["income", "expense", "transfer"],
                            "description": "The nature of the transaction"
                        },
                        "title": {
                            "type": "string",
                            "description": "A short, descriptive title (e.g., 'Bank Transfer', 'Grocery Store')"
                        },
                        "pocket": {
                            "type": "string",
                            "description": "The name of the pocket/wallet/bank that matches the notification (e.g. Gopay, Jago)"
                        },
                        "destination_pocket": {
                            "type": "string",
                            "description": "If this is a transfer, the name of the receiving pocket/wallet/bank (e.g. Gopay, Jago)"
                        },
                        "category": {
                            "type": "string",
                            "description": "The likely category for the transaction (e.g. Food, Transport)"
                        }
                    }
                }),
            },
        }];

        tracing::info!("Sending request to Ollama: Title: {}, Body: {}", title, body);

        let request = OllamaChatRequest {
            model: self.config.ollama_model.clone(),
            messages: vec![
                ChatMessage { role: "system".to_string(), content: system_prompt },
                ChatMessage { role: "user".to_string(), content: format!("Notification Title: {}\nNotification Body: {}", title, body) },
            ],
            tools,
            stream: false,
        };

        let url = format!("{}/api/chat", self.config.ollama_host);
        
        let res = self.client.post(&url)
            .header("User-Agent", "MobileMoneyBackend/1.0")
            .json(&request)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            let status = res.status();
            let err_text = res.text().await.unwrap_or_default();
            return Err(format!("Ollama API error ({}): {}", status, err_text));
        }

        tracing::info!("Ollama response: {:#?}", res);

        let body: OllamaChatResponse = res.json().await.map_err(|e| e.to_string())?;
        
        let tool_call_opt = body.message.tool_calls
            .and_then(|calls| calls.into_iter().find(|c| c.function.name == "extract_transaction"));

        let tool_call = match tool_call_opt {
            Some(call) => call,
            None => {
                tracing::info!("AI determined this is not a transaction.");
                return Ok(None);
            }
        };

        let parsed: AiParsedNotification = serde_json::from_value(tool_call.function.arguments)
            .map_err(|e| format!("Failed to parse tool arguments: {}", e))?;

        tracing::info!("Parsed notification: {:#?}", parsed);

        // Manual validation for required fields
        if parsed.is_transaction {
            if parsed.amount.is_none() {
                return Err("AI failed to extract a valid amount".to_string());
            }
            if parsed.transaction_type.is_none() {
                return Err("AI failed to determine transaction type".to_string());
            }
            if parsed.title.is_none() {
                return Err("AI failed to generate a transaction title".to_string());
            }
        }

        Ok(Some(parsed))
    }
}
