pub mod manager;

pub use manager::SessionManager;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub channel: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Session {
    pub fn new(channel: impl Into<String>, user_id: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            channel: channel.into(),
            user_id: user_id.into(),
            created_at: now,
            last_active: now,
            metadata: HashMap::new(),
        }
    }
    
    pub fn touch(&mut self) {
        self.last_active = Utc::now();
    }
}
