use super::{Tool, ToolResult, ToolParameters, ToolProperty};
use crate::Error;
use async_trait::async_trait;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct FilesystemTool {
    params: ToolParameters,
}

impl FilesystemTool {
    pub fn new() -> Self {
        let mut properties = HashMap::new();
        properties.insert("operation".to_string(), ToolProperty {
            type_: "string".to_string(),
            description: Some("Operation: read, write, list, delete".to_string()),
            default: None,
        });
        properties.insert("path".to_string(), ToolProperty {
            type_: "string".to_string(),
            description: Some("File or directory path".to_string()),
            default: None,
        });
        properties.insert("content".to_string(), ToolProperty {
            type_: "string".to_string(),
            description: Some("Content to write (for write operation)".to_string()),
            default: None,
        });
        
        Self {
            params: ToolParameters {
                properties,
                required: vec!["operation".to_string(), "path".to_string()],
            },
        }
    }
}

#[async_trait]
impl Tool for FilesystemTool {
    fn name(&self) -> &str {
        "filesystem"
    }
    
    fn description(&self) -> &str {
        "Read, write, list, or delete files in the workspace"
    }
    
    fn parameters(&self) -> &ToolParameters {
        &self.params
    }
    
    async fn execute(&self, args: &str) -> Result<ToolResult, Error> {
        let args: serde_json::Value = serde_json::from_str(args)
            .map_err(|e| Error::Tool(e.to_string()))?;
        
        let operation = args.get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("list");
        
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");
        
        let workspace = std::env::current_dir()
            .unwrap_or_else(|_| Path::new(".").to_owned());
        
        let full_path = workspace.join(path);
        
        match operation {
            "read" => {
                if !full_path.exists() {
                    return Ok(ToolResult::err("File not found"));
                }
                match fs::read_to_string(&full_path) {
                    Ok(content) => Ok(ToolResult::ok(content)),
                    Err(e) => Ok(ToolResult::err(e.to_string())),
                }
            },
            "write" => {
                let content = args.get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                if let Some(parent) = full_path.parent() {
                    fs::create_dir_all(parent).ok();
                }
                
                match fs::write(&full_path, content) {
                    Ok(_) => Ok(ToolResult::ok("File written successfully")),
                    Err(e) => Ok(ToolResult::err(e.to_string())),
                }
            },
            "list" => {
                if !full_path.is_dir() {
                    return Ok(ToolResult::err("Not a directory"));
                }
                
                match fs::read_dir(&full_path) {
                    Ok(entries) => {
                        let names: Vec<String> = entries
                            .filter_map(|e| e.ok())
                            .map(|e| {
                                let path = e.path();
                                if path.is_dir() {
                                    format!("{}/", e.file_name().to_string_lossy())
                                } else {
                                    e.file_name().to_string_lossy().to_string()
                                }
                            })
                            .collect();
                        Ok(ToolResult::ok(names.join("\n")))
                    },
                    Err(e) => Ok(ToolResult::err(e.to_string())),
                }
            },
            "delete" => {
                match fs::remove_file(&full_path).or_else(|_| fs::remove_dir_all(&full_path)) {
                    Ok(_) => Ok(ToolResult::ok("Deleted successfully")),
                    Err(e) => Ok(ToolResult::err(e.to_string())),
                }
            },
            _ => Ok(ToolResult::err("Unknown operation")),
        }
    }
}

pub struct WebSearchTool {
    params: ToolParameters,
}

impl WebSearchTool {
    pub fn new() -> Self {
        let mut properties = HashMap::new();
        properties.insert("query".to_string(), ToolProperty {
            type_: "string".to_string(),
            description: Some("Search query".to_string()),
            default: None,
        });
        properties.insert("num_results".to_string(), ToolProperty {
            type_: "number".to_string(),
            description: Some("Number of results to return".to_string()),
            default: Some(serde_json::json!(5)),
        });
        
        Self {
            params: ToolParameters {
                properties,
                required: vec!["query".to_string()],
            },
        }
    }
}

#[async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }
    
    fn description(&self) -> &str {
        "Search the web for information"
    }
    
    fn parameters(&self) -> &ToolParameters {
        &self.params
    }
    
    async fn execute(&self, args: &str) -> Result<ToolResult, Error> {
        let args: serde_json::Value = serde_json::from_str(args)
            .map_err(|e| Error::Tool(e.to_string()))?;
        
        let query = args.get("query")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        Ok(ToolResult::ok(format!(
            "Web search for '{}' requires API configuration. Use Python FFI with provider.",
            query
        )))
    }
}

pub struct MessageTool {
    params: ToolParameters,
}

impl MessageTool {
    pub fn new() -> Self {
        let mut properties = HashMap::new();
        properties.insert("channel".to_string(), ToolProperty {
            type_: "string".to_string(),
            description: Some("Channel to send message to".to_string()),
            default: None,
        });
        properties.insert("content".to_string(), ToolProperty {
            type_: "string".to_string(),
            description: Some("Message content".to_string()),
            default: None,
        });
        
        Self {
            params: ToolParameters {
                properties,
                required: vec!["content".to_string()],
            },
        }
    }
}

#[async_trait]
impl Tool for MessageTool {
    fn name(&self) -> &str {
        "message"
    }
    
    fn description(&self) -> &str {
        "Send a message to a channel"
    }
    
    fn parameters(&self) -> &ToolParameters {
        &self.params
    }
    
    async fn execute(&self, args: &str) -> Result<ToolResult, Error> {
        let args: serde_json::Value = serde_json::from_str(args)
            .map_err(|e| Error::Tool(e.to_string()))?;
        
        let content = args.get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        Ok(ToolResult::ok(format!(
            "Message queued: {}",
            content
        )))
    }
}

pub struct CronTool {
    params: ToolParameters,
}

impl CronTool {
    pub fn new() -> Self {
        let mut properties = HashMap::new();
        properties.insert("action".to_string(), ToolProperty {
            type_: "string".to_string(),
            description: Some("Action: list, add, remove".to_string()),
            default: None,
        });
        properties.insert("schedule".to_string(), ToolProperty {
            type_: "string".to_string(),
            description: Some("Cron schedule expression".to_string()),
            default: None,
        });
        properties.insert("prompt".to_string(), ToolProperty {
            type_: "string".to_string(),
            description: Some("Prompt to execute".to_string()),
            default: None,
        });
        properties.insert("name".to_string(), ToolProperty {
            type_: "string".to_string(),
            description: Some("Job name".to_string()),
            default: None,
        });
        
        Self {
            params: ToolParameters {
                properties,
                required: vec!["action".to_string()],
            },
        }
    }
}

#[async_trait]
impl Tool for CronTool {
    fn name(&self) -> &str {
        "cron"
    }
    
    fn description(&self) -> &str {
        "Manage scheduled cron jobs"
    }
    
    fn parameters(&self) -> &ToolParameters {
        &self.params
    }
    
    async fn execute(&self, args: &str) -> Result<ToolResult, Error> {
        let args: serde_json::Value = serde_json::from_str(args)
            .map_err(|e| Error::Tool(e.to_string()))?;
        
        let action = args.get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("list");
        
        match action {
            "list" => Ok(ToolResult::ok("Cron jobs managed via config")),
            _ => Ok(ToolResult::ok(format!("Cron action '{}' noted", action))),
        }
    }
}
