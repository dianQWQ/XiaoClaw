pub mod agent;
pub mod config;
pub mod memory;
pub mod tools;
pub mod session;
pub mod ffi;

pub use agent::{Agent, Message, Response, MessageRole, ToolCall, LLMProvider};
pub use config::{Config, ProviderConfig, ChannelConfig, AgentConfig};
pub use memory::MemoryStore;
pub use tools::{Tool, ToolResult, ToolRegistry};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Agent error: {0}")]
    Agent(String),
    
    #[error("Tool error: {0}")]
    Tool(String),
    
    #[error("Memory error: {0}")]
    Memory(String),
    
    #[error("Session error: {0}")]
    Session(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
}

#[cfg(feature = "python")]
impl From<pyo3::PyErr> for Error {
    fn from(e: pyo3::PyErr) -> Self {
        Error::Agent(e.to_string())
    }
}
