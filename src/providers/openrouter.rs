use super::{ChatResponse, Usage};
use crate::{Message, Response, Error, agent::ToolCall};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct OpenRouterProvider {
    client: Client,
    api_key: String,
}

impl OpenRouterProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }
    
    async fn request(&self, model: &str, messages: &[Message], temperature: Option<f32>, max_tokens: Option<u32>) -> Result<ChatResponse, Error> {
        let url = "https://openrouter.ai/api/v1/chat/completions";
        
        let msgs: Vec<serde_json::Value> = messages.iter().map(|m| {
            serde_json::json!({
                "role": match m.role {
                    crate::agent::MessageRole::System => "system",
                    crate::agent::MessageRole::User => "user",
                    crate::agent::MessageRole::Assistant => "assistant",
                    crate::agent::MessageRole::Tool => "tool",
                },
                "content": m.content,
            })
        }).collect();
        
        let mut body = serde_json::json!({
            "model": model,
            "messages": msgs,
        });
        
        if let Some(temp) = temperature {
            body["temperature"] = serde_json::json!(temp);
        }
        
        if let Some(tokens) = max_tokens {
            body["max_tokens"] = serde_json::json!(tokens);
        }
        
        let response = self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", "https://github.com/dianQWQ/XiaoClaw")
            .header("X-Title", "XiaoClaw")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Agent(e.to_string()))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Agent(format!("OpenRouter API error {}: {}", status, text)));
        }
        
        let resp: OpenRouterResponse = response.json().await.map_err(|e| Error::Agent(e.to_string()))?;
        
        let choice = resp.choices.first().ok_or_else(|| Error::Agent("No response choices".to_string()))?;
        
        let content = choice.message.content.clone().unwrap_or_default();
        
        let tool_calls: Vec<ToolCall> = choice.message.tool_calls.iter().map(|c| {
            let args = serde_json::from_str(&c.function.arguments).unwrap_or(serde_json::json!({}));
            ToolCall {
                id: c.id.clone(),
                name: c.function.name.clone(),
                arguments: args,
            }
        }).collect();
        
        Ok(ChatResponse {
            content,
            tool_calls,
            usage: None,
        })
    }
}

#[async_trait]
impl super::LLMProvider for OpenRouterProvider {
    async fn chat(&self, model: &str, messages: &[Message]) -> Result<Response, Error> {
        let resp = self.request(model, messages, None, None).await?;
        
        Ok(Response {
            content: resp.content,
            tool_calls: resp.tool_calls,
            done: true,
        })
    }
    
    async fn chat_streaming(&self, _model: &str, _messages: &[Message], _on_chunk: Box<dyn Fn(String) + Send + Sync>) -> Result<Response, Error> {
        Err(Error::Agent("Streaming not implemented yet".to_string()))
    }
}

#[derive(Debug, Deserialize)]
struct OpenRouterResponse {
    id: String,
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: MessageResponse,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MessageResponse {
    role: String,
    content: Option<String>,
    #[serde(default)]
    tool_calls: Vec<ToolCallInfo>,
}

#[derive(Debug, Deserialize)]
struct ToolCallInfo {
    id: String,
    #[serde(rename = "type")]
    type_: String,
    function: FunctionInfo,
}

#[derive(Debug, Deserialize)]
struct FunctionInfo {
    name: String,
    arguments: String,
}
