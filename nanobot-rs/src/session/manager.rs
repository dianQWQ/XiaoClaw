use super::Session;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

pub struct SessionManager {
    sessions: RwLock<HashMap<String, Arc<RwLock<Session>>>>,
    timeout: Duration,
}

impl SessionManager {
    pub fn new(timeout_minutes: u64) -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            timeout: Duration::from_secs(timeout_minutes * 60),
        }
    }

    pub fn get_or_create(&self, channel: &str, user_id: &str) -> Arc<RwLock<Session>> {
        let key = format!("{}:{}", channel, user_id);

        {
            let sessions = self.sessions.read();
            if let Some(session) = sessions.get(&key) {
                let mut s = session.write();
                s.touch();
                return session.clone();
            }
        }

        let session = Arc::new(RwLock::new(Session::new(channel, user_id)));
        self.sessions.write().insert(key, session.clone());
        session
    }

    pub fn get(&self, channel: &str, user_id: &str) -> Option<Arc<RwLock<Session>>> {
        let key = format!("{}:{}", channel, user_id);
        self.sessions.read().get(&key).cloned()
    }

    pub fn remove(&self, channel: &str, user_id: &str) -> bool {
        let key = format!("{}:{}", channel, user_id);
        self.sessions.write().remove(&key).is_some()
    }

    pub fn list(&self) -> Vec<Session> {
        self.sessions
            .read()
            .values()
            .map(|s| s.read().clone())
            .collect()
    }

    pub fn cleanup_expired(&self) -> usize {
        let now = chrono::Utc::now();
        let mut to_remove = Vec::new();

        {
            let sessions = self.sessions.read();
            for (key, session) in sessions.iter() {
                let s = session.read();
                if now.signed_duration_since(s.last_active)
                    > chrono::Duration::from_std(self.timeout).unwrap()
                {
                    to_remove.push(key.clone());
                }
            }
        }

        let mut sessions = self.sessions.write();
        for key in &to_remove {
            sessions.remove(key);
        }

        to_remove.len()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new(30)
    }
}
