pub mod context;
pub mod loop_core;

pub use context::ContextBuilder;
pub use loop_core::AgentLoop;
pub use crate::providers::LLMProvider;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub done: bool,
}

pub struct Agent {
    loop_core: Arc<AgentLoop>,
}

impl Agent {
    pub fn new(
        agent_config: super::config::AgentConfig,
        tools: Arc<super::tools::ToolRegistry>,
    ) -> Self {
        let config = Arc::new(RwLock::new(agent_config));
        let loop_core = Arc::new(AgentLoop::new(config, tools));
        
        Self {
            loop_core,
        }
    }
    
    pub fn set_provider(&self, provider: Arc<dyn LLMProvider + Send + Sync>) {
        self.loop_core.set_provider(provider);
    }
    
    pub async fn process(&self, input: &str) -> Result<Response, super::Error> {
        self.loop_core.run(input).await
    }
    
    pub fn set_system_prompt(&self, prompt: String) {
        self.loop_core.set_system_prompt(prompt);
    }
    
    pub fn clear_messages(&self) {
        self.loop_core.clear_messages();
    }
}
