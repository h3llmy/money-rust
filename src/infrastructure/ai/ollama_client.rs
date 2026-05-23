use serde::{Deserialize, Serialize};
use crate::core::config::Config;
use std::sync::Arc;

#[derive(Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    tools: Vec<Tool>,
    stream: bool,
    think: bool,
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
            think: false,
        };

        let url = format!("{}/api/chat", self.config.ollama_host);
        
        let res = self.client.post(&url)
            .header("User-Agent", "MobileMoneyBackend/1.0")
            .timeout(std::time::Duration::from_secs(15))
            .json(&request)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            let status = res.status();
            let err_text = res.text().await.unwrap_or_default();
            return Err(format!("Ollama API error ({}): {}", status, err_text));
        }

        let text = res.text().await.map_err(|e| e.to_string())?;
        tracing::info!("Ollama parse_notification response: {}", text);

        let body: OllamaChatResponse = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        
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

    async fn parse_transaction_query(
        &self,
        query: &str,
        current_date: &str,
    ) -> Result<crate::infrastructure::ai::AiTransactionQuery, String> {
        let system_prompt = format!(
            "You are a financial query parser. Parse the user's request and extract filtering parameters for querying a transaction database.\n\
            Current date and time is {}.\n\
            Extract start_date, end_date (in ISO 8601 format, e.g. 2026-05-01T00:00:00Z), category_name, pocket_name, transaction_type (income, expense, transfer), and limit (number).\n\
            If no specific date is mentioned, do not provide dates. E.g. 'last month', 'this week', 'yesterday' should be translated to exact ISO 8601 bounds.",
            current_date
        );

        let tools = vec![Tool {
            tool_type: "function".to_string(),
            function: ToolFunction {
                name: "query_transactions".to_string(),
                description: "Filter transactions based on user query".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "pocket_name": { "type": "string", "description": "Pocket or wallet name (e.g. Gopay, Jago)" },
                        "category_name": { "type": "string", "description": "Category name (e.g. Food, Transport)" },
                        "start_date": { "type": "string", "description": "Start date in ISO 8601 (e.g. 2026-05-01T00:00:00Z)" },
                        "end_date": { "type": "string", "description": "End date in ISO 8601 (e.g. 2026-05-31T23:59:59Z)" },
                        "transaction_type": { "type": "string", "description": "Must be 'income', 'expense', or 'transfer'" },
                        "limit": { "type": "integer", "description": "Number of transactions to return" }
                    }
                }),
            },
        }];

        let request = OllamaChatRequest {
            model: self.config.ollama_model.clone(),
            messages: vec![
                ChatMessage { role: "system".to_string(), content: system_prompt },
                ChatMessage { role: "user".to_string(), content: query.to_string() },
            ],
            tools,
            stream: false,
            think: false,
        };

        let url = format!("{}/api/chat", self.config.ollama_host);
        
        let res = self.client.post(&url)
            .header("User-Agent", "MobileMoneyBackend/1.0")
            .timeout(std::time::Duration::from_secs(15))
            .json(&request)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            let status = res.status();
            let err_text = res.text().await.unwrap_or_default();
            return Err(format!("Ollama API error ({}): {}", status, err_text));
        }

        let text = res.text().await.map_err(|e| e.to_string())?;
        tracing::info!("Ollama parse_transaction_query response: {}", text);

        let body: OllamaChatResponse = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        
        let tool_call_opt = body.message.tool_calls
            .and_then(|calls| calls.into_iter().find(|c| c.function.name == "query_transactions"));

        let tool_call = match tool_call_opt {
            Some(call) => call,
            None => return Ok(crate::infrastructure::ai::AiTransactionQuery::default()),
        };

        let parsed: crate::infrastructure::ai::AiTransactionQuery = serde_json::from_value(tool_call.function.arguments)
            .map_err(|e| format!("Failed to parse tool arguments: {}", e))?;

        Ok(parsed)
    }

    async fn analyze_transactions(
        &self,
        transactions_json: &str,
        user_query: Option<&str>,
    ) -> Result<futures_util::stream::BoxStream<'static, Result<String, String>>, String> {
        let system_prompt = "You are an expert personal finance AI assistant. Analyze the user's transactions provided in JSON format and respond to their query. Provide rich, actionable, and visually appealing financial analysis, breakdown of spending/income patterns, and personalized recommendations. Use clean and well-structured Markdown.";
        let query = user_query.unwrap_or("Analyze my transactions, provide an overview of my financial status, spending habits, and actionable recommendations.");

        let request = OllamaChatRequest {
            model: self.config.ollama_model.clone(),
            messages: vec![
                ChatMessage { role: "system".to_string(), content: system_prompt.to_string() },
                ChatMessage { role: "user".to_string(), content: format!("Transactions JSON:\n{}\n\nUser Query: {}", transactions_json, query) },
            ],
            tools: vec![],
            stream: true,
            think: false,
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

        let stream = futures_util::stream::unfold((res.bytes_stream(), String::new()), |(mut byte_stream, mut buffer)| async move {
            use futures_util::StreamExt;
            loop {
                if let Some(pos) = buffer.find('\n') {
                    let line = buffer[..pos].to_string();
                    buffer = buffer[pos+1..].to_string();
                    
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&line) {
                        if let Some(message) = parsed.get("message") {
                            if let Some(content) = message.get("content").and_then(|c| c.as_str()) {
                                if !content.is_empty() {
                                    return Some((Ok(content.to_string()), (byte_stream, buffer)));
                                }
                            }
                        }
                    }
                    continue;
                }

                match byte_stream.next().await {
                    Some(Ok(bytes)) => {
                        if let Ok(s) = std::str::from_utf8(&bytes) {
                            buffer.push_str(s);
                        }
                    },
                    Some(Err(e)) => return Some((Err(e.to_string()), (byte_stream, buffer))),
                    None => {
                        if !buffer.is_empty() {
                            let content = buffer.clone();
                            buffer.clear();
                            // Optional: one last check if there's any JSON in the buffer
                            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
                                if let Some(message) = parsed.get("message") {
                                    if let Some(content) = message.get("content").and_then(|c| c.as_str()) {
                                        if !content.is_empty() {
                                            return Some((Ok(content.to_string()), (byte_stream, buffer)));
                                        }
                                    }
                                }
                            }
                        }
                        return None;
                    }
                }
            }
        });

        use futures_util::StreamExt;
        Ok(stream.boxed())
    }
}
