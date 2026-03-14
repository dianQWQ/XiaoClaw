use crate::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub workspace: Option<String>,
    pub providers: HashMap<String, ProviderConfig>,
    pub agents: HashMap<String, AgentConfig>,
    pub channels: HashMap<String, ChannelConfig>,
    pub tools: ToolsConfig,
    #[serde(default)]
    pub memory: MemoryConfig,
    #[serde(default)]
    pub cron: CronConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            workspace: Some("./workspace".to_string()),
            providers: HashMap::new(),
            agents: HashMap::from([("defaults".to_string(), AgentConfig::default())]),
            channels: HashMap::new(),
            tools: ToolsConfig::default(),
            memory: MemoryConfig::default(),
            cron: CronConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub model: Option<String>,
    #[serde(default)]
    pub options: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub model: String,
    pub provider: String,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub system_prompt: Option<String>,
    #[serde(default)]
    pub tools: Vec<String>,
}

fn default_temperature() -> f32 {
    0.7
}
fn default_max_tokens() -> Option<u32> {
    Some(4096)
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            model: "claude-3-haiku".to_string(),
            provider: "openai".to_string(),
            temperature: 0.7,
            max_tokens: Some(4096),
            system_prompt: None,
            tools: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub allow_from: Vec<String>,
    #[serde(default)]
    pub options: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsConfig {
    #[serde(default = "default_restrict_workspace")]
    pub restrict_to_workspace: bool,
    #[serde(default)]
    pub web: WebToolsConfig,
    #[serde(default)]
    pub shell: ShellToolsConfig,
    #[serde(default)]
    pub mcp_servers: HashMap<String, McpServerConfig>,
}

fn default_restrict_workspace() -> bool {
    true
}

impl Default for ToolsConfig {
    fn default() -> Self {
        Self {
            restrict_to_workspace: true,
            web: WebToolsConfig::default(),
            shell: ShellToolsConfig::default(),
            mcp_servers: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebToolsConfig {
    #[serde(default)]
    pub search: Option<SearchConfig>,
}

impl Default for WebToolsConfig {
    fn default() -> Self {
        Self { search: None }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    pub provider: String,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellToolsConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
}

fn default_timeout() -> u64 {
    30
}

impl Default for ShellToolsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            timeout_seconds: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub command: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    #[serde(default = "default_max_messages")]
    pub max_messages: usize,
    #[serde(default)]
    pub persist: bool,
}

fn default_max_messages() -> usize {
    100
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            max_messages: 100,
            persist: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CronConfig {
    #[serde(default)]
    pub jobs: Vec<CronJob>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJob {
    pub name: String,
    pub schedule: String,
    pub prompt: String,
    #[serde(default)]
    pub channels: Vec<String>,
}

pub fn load_config(path: &str) -> Result<Config, Error> {
    let content = std::fs::read_to_string(path)?;
    let config: Config = serde_json::from_str(&content)?;
    Ok(config)
}
