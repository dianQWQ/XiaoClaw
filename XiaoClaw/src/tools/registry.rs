use super::{Tool, ToolResult};
use crate::Error;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct ToolRegistry {
    tools: RwLock<HashMap<String, Arc<dyn Tool>>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let registry = Self {
            tools: RwLock::new(HashMap::new()),
        };
        registry.register_builtin_tools();
        registry
    }
    
    fn register_builtin_tools(&self) {
        use super::builtin::{FilesystemTool, WebSearchTool, MessageTool, CronTool};
        
        let tools: Vec<Arc<dyn Tool>> = vec![
            Arc::new(FilesystemTool::new()),
            Arc::new(WebSearchTool::new()),
            Arc::new(MessageTool::new()),
            Arc::new(CronTool::new()),
        ];
        
        for tool in tools {
            self.register(tool);
        }
    }
    
    pub fn register(&self, tool: Arc<dyn Tool>) {
        let name = tool.name().to_string();
        self.tools.write().insert(name, tool);
    }
    
    pub async fn execute(&self, name: &str, args: &serde_json::Value) -> Result<ToolResult, Error> {
        let tools = self.tools.read();
        
        let tool = tools.get(name)
            .ok_or_else(|| Error::Tool(format!("Tool not found: {}", name)))?;
        
        let args_str = serde_json::to_string(args)
            .map_err(|e| Error::Tool(e.to_string()))?;
        
        tool.execute(&args_str).await
    }
    
    pub fn list_tools(&self) -> Vec<ToolInfo> {
        self.tools.read()
            .values()
            .map(|t| ToolInfo {
                name: t.name().to_string(),
                description: t.description().to_string(),
                parameters: t.parameters().clone(),
            })
            .collect()
    }
    
    pub fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.read().get(name).cloned()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::ToolParameters;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub parameters: ToolParameters,
}
