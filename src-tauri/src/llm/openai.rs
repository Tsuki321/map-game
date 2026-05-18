use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};

use super::provider::{ChatMessage, ChatResponse, FunctionCall, LlmProvider, ToolCall, ToolDefinition, UsageInfo};

/// OpenAI chat completions provider.
pub struct OpenAiProvider {
    client: Client,
    api_key: String,
    model: String,
    endpoint: String,
}

impl OpenAiProvider {
    /// Create a new OpenAI provider.
    ///
    /// * `api_key` - OpenAI API key.
    /// * `model` - Model name (e.g. "gpt-4", "gpt-3.5-turbo").
    /// * `endpoint` - Optional custom endpoint base URL. Defaults to `https://api.openai.com/v1`.
    pub fn new(api_key: String, model: String, endpoint: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model,
            endpoint: endpoint.unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
        }
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn chat(
        &self,
        messages: Vec<ChatMessage>,
        tools: Option<Vec<ToolDefinition>>,
    ) -> Result<ChatResponse, String> {
        let url = format!("{}/chat/completions", self.endpoint);

        let mut body = json!({
            "model": self.model,
            "messages": messages,
            "temperature": 0.7,
            "max_tokens": 4096,
        });

        if let Some(ref tool_defs) = tools {
            let openai_tools: Vec<Value> = tool_defs
                .iter()
                .map(|t| {
                    json!({
                        "type": "function",
                        "function": {
                            "name": t.name,
                            "description": t.description,
                            "parameters": t.parameters,
                        }
                    })
                })
                .collect();
            body["tools"] = json!(openai_tools);
            body["tool_choice"] = json!("auto");
        }

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("OpenAI request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            return Err(format!("OpenAI API error ({}): {}", status, error_text));
        }

        let json: Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse OpenAI response: {}", e))?;

        let choices = json["choices"]
            .as_array()
            .ok_or_else(|| "No choices in OpenAI response".to_string())?;

        let first_choice = choices
            .first()
            .ok_or_else(|| "Empty choices array in OpenAI response".to_string())?;

        let message = &first_choice["message"];

        let content = message["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let tool_calls: Option<Vec<ToolCall>> = if let Some(tc_array) = message["tool_calls"].as_array() {
            let parsed: Vec<ToolCall> = tc_array
                .iter()
                .map(|tc| ToolCall {
                    id: tc["id"].as_str().unwrap_or("").to_string(),
                    call_type: tc["type"].as_str().unwrap_or("function").to_string(),
                    function: FunctionCall {
                        name: tc["function"]["name"].as_str().unwrap_or("").to_string(),
                        arguments: tc["function"]["arguments"].as_str().unwrap_or("{}").to_string(),
                    },
                })
                .collect();
            Some(parsed)
        } else {
            None
        };

        let usage = json.get("usage").map(|u| UsageInfo {
            prompt_tokens: u["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: u["completion_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: u["total_tokens"].as_u64().unwrap_or(0) as u32,
        });

        Ok(ChatResponse {
            message: ChatMessage {
                role: message["role"].as_str().unwrap_or("assistant").to_string(),
                content,
                tool_calls,
                tool_call_id: None,
                name: None,
            },
            usage,
        })
    }
}
