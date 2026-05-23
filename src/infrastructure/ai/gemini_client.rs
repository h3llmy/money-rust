use serde::{Deserialize, Serialize};
use crate::core::config::Config;
use std::sync::Arc;
use super::{AiClient, AiParsedNotification};
use async_trait::async_trait;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiRequest {
    system_instruction: Content,
    contents: Vec<Content>,
    tools: Vec<Tool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_config: Option<ToolConfig>,
}

#[derive(Serialize, Deserialize)]
struct Content {
    parts: Vec<Part>,
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Part {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    function_call: Option<FunctionCall>,
}

#[derive(Serialize, Deserialize)]
struct FunctionCall {
    name: String,
    args: serde_json::Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Tool {
    function_declarations: Vec<FunctionDeclaration>,
}

#[derive(Serialize)]
struct FunctionDeclaration {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ToolConfig {
    function_calling_config: FunctionCallingConfig,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FunctionCallingConfig {
    mode: String,
    allowed_function_names: Vec<String>,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<Candidate>>,
}

#[derive(Deserialize)]
struct Candidate {
    content: Content,
}

pub struct GeminiClient {
    config: Arc<Config>,
    client: reqwest::Client,
}

impl GeminiClient {
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl AiClient for GeminiClient {
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
            function_declarations: vec![FunctionDeclaration {
                name: "extract_transaction".to_string(),
                description: "Extract financial transaction details from a notification message".to_string(),
                parameters: serde_json::json!({
                    "type": "OBJECT",
                    "properties": {
                        "is_transaction": {
                            "type": "BOOLEAN",
                            "description": "True if the notification is a financial transaction (income, expense, transfer), false otherwise"
                        },
                        "amount": {
                            "type": "NUMBER",
                            "description": "The transaction amount"
                        },
                        "transaction_type": {
                            "type": "STRING",
                            "description": "The nature of the transaction. Must be 'income', 'expense', or 'transfer'"
                        },
                        "title": {
                            "type": "STRING",
                            "description": "A short, descriptive title (e.g., 'Bank Transfer', 'Grocery Store')"
                        },
                        "pocket": {
                            "type": "STRING",
                            "description": "The name of the pocket/wallet/bank that matches the notification (e.g. Gopay, Jago)"
                        },
                        "destination_pocket": {
                            "type": "STRING",
                            "description": "If this is a transfer, the name of the receiving pocket/wallet/bank (e.g. Gopay, Jago)"
                        },
                        "category": {
                            "type": "STRING",
                            "description": "The likely category for the transaction (e.g. Food, Transport)"
                        }
                    },
                    "required": ["is_transaction"]
                }),
            }],
        }];

        tracing::info!("Sending request to Gemini: Title: {}, Body: {}", title, body);

        let request = GeminiRequest {
            system_instruction: Content {
                role: None,
                parts: vec![Part { text: Some(system_prompt), function_call: None }],
            },
            contents: vec![Content {
                role: Some("user".to_string()),
                parts: vec![Part { 
                    text: Some(format!("Notification Title: {}\nNotification Body: {}", title, body)),
                    function_call: None 
                }],
            }],
            tools,
            tool_config: Some(ToolConfig {
                function_calling_config: FunctionCallingConfig {
                    mode: "ANY".to_string(), // Force to call the function
                    allowed_function_names: vec!["extract_transaction".to_string()],
                }
            })
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.config.gemini_model, self.config.gemini_api_key
        );
        
        let res = self.client.post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            let status = res.status();
            let err_text = res.text().await.unwrap_or_default();
            return Err(format!("Gemini API error ({}): {}", status, err_text));
        }

        let text = res.text().await.map_err(|e| e.to_string())?;
        tracing::info!("Gemini parse_notification response: {}", text);

        let body: GeminiResponse = serde_json::from_str(&text).map_err(|e| format!("Failed to parse Gemini response: {}", e))?;
        
        let candidates = body.candidates.unwrap_or_default();
        let first_candidate = candidates.into_iter().next().ok_or_else(|| "No candidates returned by Gemini".to_string())?;
        
        let function_call_opt = first_candidate.content.parts.into_iter()
            .find_map(|p| p.function_call);

        let function_call = match function_call_opt {
            Some(call) => call,
            None => {
                tracing::info!("AI determined this is not a transaction.");
                return Ok(None);
            }
        };

        if function_call.name != "extract_transaction" {
            return Err(format!("Unexpected function call: {}", function_call.name));
        }

        let parsed: AiParsedNotification = serde_json::from_value(function_call.args)
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
            function_declarations: vec![FunctionDeclaration {
                name: "query_transactions".to_string(),
                description: "Filter transactions based on user query".to_string(),
                parameters: serde_json::json!({
                    "type": "OBJECT",
                    "properties": {
                        "pocket_name": { "type": "STRING", "description": "Pocket or wallet name (e.g. Gopay, Jago)" },
                        "category_name": { "type": "STRING", "description": "Category name (e.g. Food, Transport)" },
                        "start_date": { "type": "STRING", "description": "Start date in ISO 8601 (e.g. 2026-05-01T00:00:00Z)" },
                        "end_date": { "type": "STRING", "description": "End date in ISO 8601 (e.g. 2026-05-31T23:59:59Z)" },
                        "transaction_type": { "type": "STRING", "description": "Must be 'income', 'expense', or 'transfer'" },
                        "limit": { "type": "INTEGER", "description": "Number of transactions to return" }
                    }
                }),
            }],
        }];

        let request = GeminiRequest {
            system_instruction: Content {
                role: None,
                parts: vec![Part { text: Some(system_prompt), function_call: None }],
            },
            contents: vec![Content {
                role: Some("user".to_string()),
                parts: vec![Part { 
                    text: Some(query.to_string()),
                    function_call: None 
                }],
            }],
            tools,
            tool_config: Some(ToolConfig {
                function_calling_config: FunctionCallingConfig {
                    mode: "ANY".to_string(),
                    allowed_function_names: vec!["query_transactions".to_string()],
                }
            })
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.config.gemini_model, self.config.gemini_api_key
        );
        
        let res = self.client.post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            return Err(format!("Gemini API error ({}): {}", res.status(), res.text().await.unwrap_or_default()));
        }

        let text = res.text().await.map_err(|e| e.to_string())?;
        tracing::info!("Gemini parse_transaction_query response: {}", text);

        let body: GeminiResponse = serde_json::from_str(&text).map_err(|e| format!("Failed to parse Gemini response: {}", e))?;
        let candidates = body.candidates.unwrap_or_default();
        let first_candidate = candidates.into_iter().next().ok_or_else(|| "No candidates returned by Gemini".to_string())?;
        
        let function_call_opt = first_candidate.content.parts.into_iter()
            .find_map(|p| p.function_call);

        let function_call = match function_call_opt {
            Some(call) => call,
            None => return Ok(crate::infrastructure::ai::AiTransactionQuery::default()),
        };

        let parsed: crate::infrastructure::ai::AiTransactionQuery = serde_json::from_value(function_call.args)
            .map_err(|e| format!("Failed to parse tool args: {}", e))?;

        Ok(parsed)
    }

    async fn analyze_transactions(
        &self,
        transactions_json: &str,
        user_query: Option<&str>,
    ) -> Result<futures_util::stream::BoxStream<'static, Result<String, String>>, String> {
        let system_prompt = "You are an expert personal finance AI assistant. Analyze the user's transactions provided in JSON format and respond to their query. Provide rich, actionable, and visually appealing financial analysis, breakdown of spending/income patterns, and personalized recommendations. Use clean and well-structured Markdown.";
        let query = user_query.unwrap_or("Analyze my transactions, provide an overview of my financial status, spending habits, and actionable recommendations.");

        let request = GeminiRequest {
            system_instruction: Content {
                role: None,
                parts: vec![Part { text: Some(system_prompt.to_string()), function_call: None }],
            },
            contents: vec![Content {
                role: Some("user".to_string()),
                parts: vec![Part { 
                    text: Some(format!("Transactions JSON:\n{}\n\nUser Query: {}", transactions_json, query)),
                    function_call: None 
                }],
            }],
            tools: vec![],
            tool_config: None,
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?alt=sse&key={}",
            self.config.gemini_model, self.config.gemini_api_key
        );
        
        let res = self.client.post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            let status = res.status();
            let err_text = res.text().await.unwrap_or_default();
            return Err(format!("Gemini API error ({}): {}", status, err_text));
        }

        let stream = futures_util::stream::unfold((res.bytes_stream(), String::new()), |(mut byte_stream, mut buffer)| async move {
            use futures_util::StreamExt;
            loop {
                if let Some(pos) = buffer.find("\n\n") {
                    let chunk = buffer[..pos].to_string();
                    buffer = buffer[pos+2..].to_string();
                    
                    if chunk.starts_with("data: ") {
                        let json_str = &chunk[6..];
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_str) {
                            if let Some(candidates) = parsed.get("candidates").and_then(|c| c.as_array()) {
                                if let Some(first) = candidates.first() {
                                    if let Some(parts) = first.get("content").and_then(|c| c.get("parts")).and_then(|p| p.as_array()) {
                                        if let Some(part) = parts.first() {
                                            if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                                                if !text.is_empty() {
                                                    return Some((Ok(text.to_string()), (byte_stream, buffer)));
                                                }
                                            }
                                        }
                                    }
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
                        if !buffer.is_empty() && buffer.starts_with("data: ") {
                            let json_str = &buffer[6..];
                            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_str) {
                                if let Some(candidates) = parsed.get("candidates").and_then(|c| c.as_array()) {
                                    if let Some(first) = candidates.first() {
                                        if let Some(parts) = first.get("content").and_then(|c| c.get("parts")).and_then(|p| p.as_array()) {
                                            if let Some(part) = parts.first() {
                                                if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                                                    if !text.is_empty() {
                                                        buffer.clear();
                                                        return Some((Ok(text.to_string()), (byte_stream, buffer)));
                                                    }
                                                }
                                            }
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
