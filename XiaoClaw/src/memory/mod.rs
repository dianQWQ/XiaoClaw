pub mod store;

pub use store::MemoryStore;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

impl MemoryEntry {
    pub fn new(session_id: impl Into<String>, role: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            session_id: session_id.into(),
            role: role.into(),
            content: content.into(),
            created_at: Utc::now(),
            metadata: None,
        }
    }
}
