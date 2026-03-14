use crate::{config::AgentConfig, tools::ToolRegistry, Error, Message, MessageRole, Response, ToolCall};
use parking_lot::RwLock;
use std::sync::Arc;
use std::future::Future;
use std::pin::Pin;

const MAX_ITERATIONS: usize = 20;

pub trait LLMProvider: Send + Sync {
    fn chat(&self, model: &str, messages: &[Message]) -> Pin<Box<dyn Future<Output = Result<Response, Error>> + Send + '_>>;
}

pub struct AgentLoop {
    config: Arc<RwLock<AgentConfig>>,
    tools: Arc<ToolRegistry>,
    messages: RwLock<Vec<Message>>,
    system_prompt: RwLock<Option<String>>,
    provider: Arc<dyn LLMProvider + Send + Sync>,
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
            provider: Arc::new(DefaultProvider),
        }
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
        
        self.provider.chat(&config.model, &all_messages).await
    }
    
    async fn execute_tool(&self, tool_call: &ToolCall) -> Result<String, Error> {
        let result = self.tools.execute(&tool_call.name, &tool_call.arguments).await?;
        Ok(result.content)
    }
}

impl LLMProvider for DefaultProvider {
    fn chat(&self, _model: &str, _messages: &[Message]) -> Pin<Box<dyn Future<Output = Result<Response, Error>> + Send + '_>> {
        Box::pin(async {
            Err(Error::Agent("No LLM provider configured. Use Python FFI to set provider.".to_string()))
        })
    }
}

struct DefaultProvider;
