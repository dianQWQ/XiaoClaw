use super::{ChatResponse, Usage};
use crate::{Message, Response, Error, agent::ToolCall};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct ZhipuProvider {
    client: Client,
    api_key: String,
    base_url: String,
}

impl ZhipuProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://open.bigmodel.cn/api/paas/v4".to_string(),
        }
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
            return Err(Error::Agent(format!("Zhipu API error {}: {}", status, text)));
        }
        
        let resp: ZhipuResponse = response.json().await.map_err(|e| Error::Agent(e.to_string()))?;
        
        let choice = resp.choices.first().ok_or_else(|| Error::Agent("No response choices".to_string()))?;
        
        let content = choice.message.content.clone().unwrap_or_default();
        
        Ok(ChatResponse {
            content,
            tool_calls: vec![],
            usage: None,
        })
    }
}

#[async_trait]
impl super::LLMProvider for ZhipuProvider {
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
struct ZhipuResponse {
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
}
