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

        let body: GeminiResponse = res.json().await.map_err(|e| format!("Failed to parse Gemini response: {}", e))?;
        
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
}
