use crate::{config::AgentConfig, tools::ToolRegistry, Error, Message, MessageRole, Response, ToolCall, providers::LLMProvider};
use parking_lot::RwLock;
use std::sync::Arc;
use async_trait::async_trait;

const MAX_ITERATIONS: usize = 20;

pub struct AgentLoop {
    config: Arc<RwLock<AgentConfig>>,
    tools: Arc<ToolRegistry>,
    messages: RwLock<Vec<Message>>,
    system_prompt: RwLock<Option<String>>,
    provider: RwLock<Arc<dyn LLMProvider + Send + Sync>>,
}

impl AgentLoop {
    pub fn new(
        config: Arc<RwLock<AgentConfig>>,
        tools: Arc<ToolRegistry>,
    ) -> Self {
        Self {
            config,
            tools,
            messages: RwLock::new(Vec::new()),
            system_prompt: RwLock::new(None),
            provider: RwLock::new(Arc::new(DefaultProvider)),
        }
    }
    
    pub fn set_provider(&self, provider: Arc<dyn LLMProvider + Send + Sync>) {
        *self.provider.write() = provider;
    }
    
    pub fn set_system_prompt(&self, prompt: String) {
        *self.system_prompt.write() = Some(prompt);
    }
    
    pub fn clear_messages(&self) {
        self.messages.write().clear();
    }
    
    pub async fn run(&self, input: &str) -> Result<Response, Error> {
        {
            let mut messages = self.messages.write();
            messages.push(Message {
                role: MessageRole::User,
                content: input.to_string(),
                tool_calls: None,
                tool_call_id: None,
            });
        }
        
        for iteration in 0..MAX_ITERATIONS {
            let response = self.call_llm().await?;
            
            if response.tool_calls.is_empty() {
                return Ok(response);
            }
            
            for tool_call in &response.tool_calls {
                let result = self.execute_tool(tool_call).await?;
                
                let mut messages = self.messages.write();
                messages.push(Message {
                    role: MessageRole::Tool,
                    content: result,
                    tool_calls: None,
                    tool_call_id: Some(tool_call.id.clone()),
                });
            }
            
            if iteration >= MAX_ITERATIONS - 1 {
                return Err(Error::Agent("Max iterations reached".to_string()));
            }
        }
        
        Ok(Response {
            content: String::new(),
            tool_calls: vec![],
            done: true,
        })
    }
    
    async fn call_llm(&self) -> Result<Response, Error> {
        let config = self.config.read().clone();
        let system_prompt = self.system_prompt.read().clone();
        let messages = self.messages.read().clone();
        let provider = self.provider.read().clone();
        
        let mut all_messages = Vec::new();
        
        if let Some(system) = system_prompt {
            all_messages.push(Message {
                role: MessageRole::System,
                content: system,
                tool_calls: None,
                tool_call_id: None,
            });
        }
        
        all_messages.extend(messages);
        
        provider.chat(&config.model, &all_messages).await
    }
    
    async fn execute_tool(&self, tool_call: &ToolCall) -> Result<String, Error> {
        let result = self.tools.execute(&tool_call.name, &tool_call.arguments).await?;
        Ok(result.content)
    }
}

#[async_trait]
impl LLMProvider for DefaultProvider {
    async fn chat(&self, _model: &str, _messages: &[Message]) -> Result<Response, Error> {
        Err(Error::Agent("No LLM provider configured. Use OpenAI, Anthropic, or OpenRouter provider.".to_string()))
    }
    
    async fn chat_streaming(&self, _model: &str, _messages: &[Message], _on_chunk: Box<dyn Fn(String) + Send + Sync>) -> Result<Response, Error> {
        Err(Error::Agent("Streaming not supported".to_string()))
    }
}

struct DefaultProvider;
