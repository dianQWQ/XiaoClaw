pub mod openai;
pub mod anthropic;
pub mod openrouter;
pub mod zhipu;

pub use openai::OpenAIProvider;
pub use anthropic::AnthropicProvider;
pub use openrouter::OpenRouterProvider;
pub use zhipu::ZhipuProvider;

use crate::{Message, Response, Error};
use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;

#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn chat(&self, model: &str, messages: &[Message]) -> Result<Response, Error>;
    async fn chat_streaming(&self, model: &str, messages: &[Message], on_chunk: Box<dyn Fn(String) + Send + Sync>) -> Result<Response, Error>;
}

pub type ChatFuture<'a> = Pin<Box<dyn Future<Output = Result<Response, Error>> + Send + 'a>>;

pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub tools: Option<Vec<crate::tools::ToolDefinition>>,
}

pub struct ChatResponse {
    pub content: String,
    pub tool_calls: Vec<crate::agent::ToolCall>,
    pub usage: Option<Usage>,
}

pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}
