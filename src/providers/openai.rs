use super::{ChatResponse, Usage};
use crate::{Message, Response, Error, agent::ToolCall};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct OpenAIProvider {
    client: Client,
    api_key: String,
    base_url: String,
}

impl OpenAIProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
        }
    }
    
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }
    
    async fn request(&self, model: &str, messages: &[Message], temperature: Option<f32>, max_tokens: Option<u32>) -> Result<ChatResponse, Error> {
        let url = format!("{}/chat/completions", self.base_url);
        
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
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Agent(e.to_string()))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Agent(format!("OpenAI API error {}: {}", status, text)));
        }
        
        let resp: OpenAIResponse = response.json().await.map_err(|e| Error::Agent(e.to_string()))?;
        
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
        
        let usage = resp.usage.map(|u| Usage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        });
        
        Ok(ChatResponse {
            content,
            tool_calls,
            usage,
        })
    }
}

#[async_trait]
impl super::LLMProvider for OpenAIProvider {
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
struct OpenAIResponse {
    id: String,
    choices: Vec<Choice>,
    usage: Option<UsageInfo>,
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

#[derive(Debug, Deserialize)]
struct UsageInfo {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}
