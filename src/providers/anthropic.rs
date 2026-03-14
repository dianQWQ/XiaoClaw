use super::{ChatResponse, Usage};
use crate::{Message, Response, Error, agent::ToolCall};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    base_url: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.anthropic.com/v1".to_string(),
        }
    }
    
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }
    
    async fn request(&self, model: &str, messages: &[Message], temperature: Option<f32>, max_tokens: Option<u32>) -> Result<ChatResponse, Error> {
        let url = format!("{}/messages", self.base_url);
        
        let msgs: Vec<serde_json::Value> = messages.iter().filter(|m| m.role != crate::agent::MessageRole::System).map(|m| {
            serde_json::json!({
                "role": match m.role {
                    crate::agent::MessageRole::System => "user",
                    crate::agent::MessageRole::User => "user",
                    crate::agent::MessageRole::Assistant => "assistant",
                    crate::agent::MessageRole::Tool => "user",
                },
                "content": m.content,
            })
        }).collect();
        
        let system = messages.iter()
            .filter(|m| m.role == crate::agent::MessageRole::System)
            .map(|m| m.content.clone())
            .collect::<Vec<_>>()
            .join("\n");
        
        let mut body = serde_json::json!({
            "model": model,
            "messages": msgs,
            "max_tokens": max_tokens.unwrap_or(4096),
        });
        
        if !system.is_empty() {
            body["system"] = serde_json::json!(system);
        }
        
        if let Some(temp) = temperature {
            body["temperature"] = serde_json::json!(temp);
        }
        
        let response = self.client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Agent(e.to_string()))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Agent(format!("Anthropic API error {}: {}", status, text)));
        }
        
        let resp: AnthropicResponse = response.json().await.map_err(|e| Error::Agent(e.to_string()))?;
        
        let content = resp.content.iter()
            .filter_map(|c| match c {
                ContentBlock::Text { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n");
        
        let tool_calls = resp.content.iter()
            .filter_map(|c| match c {
                ContentBlock::ToolUse { id, input, name, .. } => {
                    Some(ToolCall {
                        id: id.clone(),
                        name: name.clone(),
                        arguments: input.clone(),
                    })
                },
                _ => None,
            })
            .collect();
        
        let usage = Some(Usage {
            prompt_tokens: resp.usage.input_tokens,
            completion_tokens: resp.usage.output_tokens,
            total_tokens: resp.usage.input_tokens + resp.usage.output_tokens,
        });
        
        Ok(ChatResponse {
            content,
            tool_calls,
            usage,
        })
    }
}

#[async_trait]
impl super::LLMProvider for AnthropicProvider {
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
struct AnthropicResponse {
    id: String,
    #[serde(rename = "type")]
    type_: String,
    role: String,
    content: Vec<ContentBlock>,
    model: String,
    stop_reason: Option<String>,
    usage: UsageInfo,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ContentBlock {
    Text { 
        #[serde(rename = "type")]
        type_: String,
        text: String,
    },
    ToolUse {
        #[serde(rename = "type")]
        type_: String,
        id: String,
        name: String,
        input: serde_json::Value,
    },
}

#[derive(Debug, Deserialize)]
struct UsageInfo {
    #[serde(rename = "input_tokens")]
    input_tokens: u32,
    #[serde(rename = "output_tokens")]
    output_tokens: u32,
}
