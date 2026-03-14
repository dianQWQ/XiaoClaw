use super::MemoryEntry;
use crate::Error;
use parking_lot::Mutex;
use rusqlite::{params, Connection};
use std::sync::Arc;

pub struct MemoryStore {
    conn: Arc<Mutex<Connection>>,
    max_messages: usize,
}

impl MemoryStore {
    pub fn new(db_path: &str, max_messages: usize) -> Result<Self, Error> {
        let conn = Connection::open(db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS memories (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL,
                metadata TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_session ON memories(session_id)",
            [],
        )?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            max_messages,
        })
    }

    pub fn add(&self, entry: MemoryEntry) -> Result<(), Error> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO memories (id, session_id, role, content, created_at, metadata) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                entry.id,
                entry.session_id,
                entry.role,
                entry.content,
                entry.created_at.to_rfc3339(),
                entry.metadata.as_ref().map(|m| m.to_string()),
            ],
        )?;

        self.prune(&entry.session_id)?;

        Ok(())
    }

    pub fn get_session(
        &self,
        session_id: &str,
        limit: Option<usize>,
    ) -> Result<Vec<MemoryEntry>, Error> {
        let conn = self.conn.lock();
        let limit = limit.unwrap_or(self.max_messages);

        let mut stmt = conn.prepare(
            "SELECT id, session_id, role, content, created_at, metadata 
             FROM memories 
             WHERE session_id = ?1 
             ORDER BY created_at DESC 
             LIMIT ?2",
        )?;

        let entries = stmt.query_map(params![session_id, limit], |row| {
            let created_at_str: String = row.get(4)?;
            let metadata_str: Option<String> = row.get(5)?;

            Ok(MemoryEntry {
                id: row.get(0)?,
                session_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
                metadata: metadata_str.and_then(|s| serde_json::from_str(&s).ok()),
            })
        })?;

        let mut result: Vec<MemoryEntry> = entries.filter_map(|e| e.ok()).collect();
        result.reverse();
        Ok(result)
    }

    pub fn search(&self, session_id: &str, query: &str) -> Result<Vec<MemoryEntry>, Error> {
        let conn = self.conn.lock();

        let mut stmt = conn.prepare(
            "SELECT id, session_id, role, content, created_at, metadata 
             FROM memories 
             WHERE session_id = ?1 AND content LIKE ?2
             ORDER BY created_at DESC 
             LIMIT 10",
        )?;

        let pattern = format!("%{}%", query);
        let entries = stmt.query_map(params![session_id, pattern], |row| {
            let created_at_str: String = row.get(4)?;
            let metadata_str: Option<String> = row.get(5)?;

            Ok(MemoryEntry {
                id: row.get(0)?,
                session_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
                metadata: metadata_str.and_then(|s| serde_json::from_str(&s).ok()),
            })
        })?;

        Ok(entries.filter_map(|e| e.ok()).collect())
    }

    fn prune(&self, session_id: &str) -> Result<(), Error> {
        let conn = self.conn.lock();

        conn.execute(
            "DELETE FROM memories WHERE session_id = ?1 AND id NOT IN (
                SELECT id FROM memories WHERE session_id = ?1 
                ORDER BY created_at DESC LIMIT ?2
            )",
            params![session_id, self.max_messages],
        )?;

        Ok(())
    }

    pub fn clear_session(&self, session_id: &str) -> Result<(), Error> {
        let conn = self.conn.lock();
        conn.execute(
            "DELETE FROM memories WHERE session_id = ?1",
            params![session_id],
        )?;
        Ok(())
    }
}
