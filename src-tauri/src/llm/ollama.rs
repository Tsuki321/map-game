use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};

use super::provider::{ChatMessage, ChatResponse, FunctionCall, LlmProvider, ToolCall, ToolDefinition, UsageInfo};

/// Ollama chat completions provider (local LLM server).
pub struct OllamaProvider {
    client: Client,
    model: String,
    endpoint: String,
}

impl OllamaProvider {
    /// Create a new Ollama provider.
    ///
    /// * `model` - Model name (e.g. "llama3", "mistral").
    /// * `endpoint` - Optional custom endpoint URL. Defaults to `http://localhost:11434/api/chat`.
    pub fn new(model: String, endpoint: Option<String>) -> Self {
        Self {
            client: Client::new(),
            model,
            endpoint: endpoint.unwrap_or_else(|| "http://localhost:11434/api/chat".to_string()),
        }
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    async fn chat(
        &self,
        messages: Vec<ChatMessage>,
        tools: Option<Vec<ToolDefinition>>,
    ) -> Result<ChatResponse, String> {
        let mut body = json!({
            "model": self.model,
            "messages": messages,
            "stream": false,
        });

        if let Some(ref tool_defs) = tools {
            let ollama_tools: Vec<Value> = tool_defs
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
            body["tools"] = json!(ollama_tools);
        }

        let response = self
            .client
            .post(&self.endpoint)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Ollama request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            return Err(format!("Ollama API error ({}): {}", status, error_text));
        }

        let json: Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

        let message = &json["message"];

        let content = message["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        // Ollama may return tool_calls at message.tool_calls or message["tool_calls"]
        let tool_calls: Option<Vec<ToolCall>> =
            if let Some(tc_array) = message["tool_calls"].as_array() {
                let parsed: Vec<ToolCall> = tc_array
                    .iter()
                    .map(|tc| ToolCall {
                        id: tc["id"]
                            .as_str()
                            .unwrap_or("")
                            .to_string(),
                        call_type: tc["type"]
                            .as_str()
                            .unwrap_or("function")
                            .to_string(),
                        function: FunctionCall {
                            name: tc["function"]["name"]
                                .as_str()
                                .unwrap_or("")
                                .to_string(),
                            arguments: tc["function"]["arguments"]
                                .as_str()
                                .unwrap_or("{}")
                                .to_string(),
                        },
                    })
                    .collect();
                Some(parsed)
            } else {
                None
            };

        let usage = if let (Some(p), Some(c), Some(t)) = (
            json.get("prompt_eval_count").or(json.get("prompt_tokens")),
            json.get("eval_count").or(json.get("completion_tokens")),
            json.get("total_tokens"),
        ) {
            Some(UsageInfo {
                prompt_tokens: p.as_u64().unwrap_or(0) as u32,
                completion_tokens: c.as_u64().unwrap_or(0) as u32,
                total_tokens: t.as_u64().unwrap_or(0) as u32,
            })
        } else if let (Some(p), Some(c)) = (
            json.get("prompt_eval_count").or(json.get("prompt_tokens")),
            json.get("eval_count").or(json.get("completion_tokens")),
        ) {
            let p_count = p.as_u64().unwrap_or(0) as u32;
            let c_count = c.as_u64().unwrap_or(0) as u32;
            Some(UsageInfo {
                prompt_tokens: p_count,
                completion_tokens: c_count,
                total_tokens: p_count + c_count,
            })
        } else {
            None
        };

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
