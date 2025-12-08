//! Persistent conversation storage using SQLite
//!
//! Stores conversations locally so they persist across app restarts.
//! Data is stored in the user's local app data directory.

use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::session::{Message, MessageRole, SessionId, TokenUsage};

/// Error type for storage operations
#[derive(Debug)]
pub enum StorageError {
    /// SQLite error
    Sqlite(rusqlite::Error),
    /// Serialization error
    Serialization(String),
    /// IO error
    Io(std::io::Error),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sqlite(e) => write!(f, "SQLite error: {}", e),
            Self::Serialization(e) => write!(f, "Serialization error: {}", e),
            Self::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for StorageError {}

impl From<rusqlite::Error> for StorageError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Sqlite(e)
    }
}

impl From<std::io::Error> for StorageError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

/// Stored conversation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMetadata {
    /// Session ID
    pub session_id: String,
    /// Title (generated from first message or user-provided)
    pub title: String,
    /// Creation timestamp
    pub created_at: u64,
    /// Last message timestamp
    pub updated_at: u64,
    /// Message count
    pub message_count: u32,
}

/// Persistent conversation storage
pub struct ConversationStorage {
    /// SQLite connection (wrapped for async safety)
    conn: Arc<Mutex<Connection>>,
    /// Database file path
    db_path: PathBuf,
}

impl ConversationStorage {
    /// Create a new storage instance
    pub fn new() -> Result<Self, StorageError> {
        let db_path = Self::get_db_path()?;

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)?;
        let storage = Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path,
        };

        // Initialize schema
        tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(storage.init_schema())
        })?;

        tracing::info!("Conversation storage initialized at: {:?}", storage.db_path);
        Ok(storage)
    }

    /// Get the database file path
    fn get_db_path() -> Result<PathBuf, StorageError> {
        let data_dir = dirs::data_local_dir()
            .ok_or_else(|| StorageError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not find local data directory",
            )))?;

        Ok(data_dir
            .join("citrate")
            .join("conversations.db"))
    }

    /// Initialize the database schema
    async fn init_schema(&self) -> Result<(), StorageError> {
        let conn = self.conn.lock().await;

        // Create conversations table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS conversations (
                session_id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                message_count INTEGER DEFAULT 0
            )
            "#,
            [],
        )?;

        // Create messages table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                intent_json TEXT,
                tool_call_id TEXT,
                tool_name TEXT,
                tokens_json TEXT,
                metadata_json TEXT,
                FOREIGN KEY (session_id) REFERENCES conversations(session_id)
            )
            "#,
            [],
        )?;

        // Create index for faster queries
        conn.execute(
            r#"
            CREATE INDEX IF NOT EXISTS idx_messages_session
            ON messages(session_id, timestamp)
            "#,
            [],
        )?;

        Ok(())
    }

    /// Create a new conversation
    pub async fn create_conversation(&self, session_id: &SessionId, title: Option<&str>) -> Result<(), StorageError> {
        let conn = self.conn.lock().await;
        let now = Self::now();
        let title = title.unwrap_or("New Conversation");

        conn.execute(
            r#"
            INSERT OR REPLACE INTO conversations (session_id, title, created_at, updated_at, message_count)
            VALUES (?1, ?2, ?3, ?4, 0)
            "#,
            params![session_id.0, title, now, now],
        )?;

        Ok(())
    }

    /// Save a message to a conversation
    pub async fn save_message(&self, session_id: &SessionId, message: &Message) -> Result<(), StorageError> {
        let conn = self.conn.lock().await;

        // Serialize optional fields
        let intent_json = message.intent.as_ref()
            .map(|i| serde_json::to_string(i).ok())
            .flatten();
        let tokens_json = message.tokens.as_ref()
            .map(|t| serde_json::to_string(t).ok())
            .flatten();
        let metadata_json = serde_json::to_string(&message.metadata)
            .ok();

        let role_str = message.role.to_string();

        conn.execute(
            r#"
            INSERT OR REPLACE INTO messages
            (id, session_id, role, content, timestamp, intent_json, tool_call_id, tool_name, tokens_json, metadata_json)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![
                message.id,
                session_id.0,
                role_str,
                message.content,
                message.timestamp,
                intent_json,
                message.tool_call_id,
                message.tool_name,
                tokens_json,
                metadata_json,
            ],
        )?;

        // Update conversation metadata
        conn.execute(
            r#"
            UPDATE conversations
            SET updated_at = ?1, message_count = message_count + 1
            WHERE session_id = ?2
            "#,
            params![message.timestamp, session_id.0],
        )?;

        // Auto-update title from first user message if title is default
        if matches!(message.role, MessageRole::User) {
            let title: String = conn.query_row(
                "SELECT title FROM conversations WHERE session_id = ?1",
                params![session_id.0],
                |row| row.get(0),
            )?;

            if title == "New Conversation" {
                // Use first 50 chars of first user message as title
                let new_title: String = message.content.chars().take(50).collect();
                let new_title = if message.content.len() > 50 {
                    format!("{}...", new_title)
                } else {
                    new_title
                };

                conn.execute(
                    "UPDATE conversations SET title = ?1 WHERE session_id = ?2",
                    params![new_title, session_id.0],
                )?;
            }
        }

        Ok(())
    }

    /// Load all messages for a conversation
    pub async fn load_messages(&self, session_id: &SessionId) -> Result<Vec<Message>, StorageError> {
        let conn = self.conn.lock().await;

        let mut stmt = conn.prepare(
            r#"
            SELECT id, role, content, timestamp, intent_json, tool_call_id, tool_name, tokens_json, metadata_json
            FROM messages
            WHERE session_id = ?1
            ORDER BY timestamp ASC
            "#,
        )?;

        let messages = stmt.query_map(params![session_id.0], |row| {
            let role_str: String = row.get(1)?;
            let role = match role_str.as_str() {
                "user" => MessageRole::User,
                "assistant" => MessageRole::Assistant,
                "system" => MessageRole::System,
                "tool" => MessageRole::Tool,
                "tool_result" => MessageRole::ToolResult,
                _ => MessageRole::User,
            };

            let intent_json: Option<String> = row.get(4)?;
            let intent = intent_json.and_then(|j| serde_json::from_str(&j).ok());

            let tokens_json: Option<String> = row.get(7)?;
            let tokens = tokens_json.and_then(|j| serde_json::from_str(&j).ok());

            let metadata_json: Option<String> = row.get(8)?;
            let metadata = metadata_json
                .and_then(|j| serde_json::from_str(&j).ok())
                .unwrap_or_default();

            Ok(Message {
                id: row.get(0)?,
                role,
                content: row.get(2)?,
                timestamp: row.get(3)?,
                intent,
                tool_call_id: row.get(5)?,
                tool_name: row.get(6)?,
                tokens,
                is_streaming: false,
                metadata,
            })
        })?;

        let mut result = Vec::new();
        for msg in messages {
            result.push(msg?);
        }

        Ok(result)
    }

    /// List all conversations
    pub async fn list_conversations(&self) -> Result<Vec<ConversationMetadata>, StorageError> {
        let conn = self.conn.lock().await;

        let mut stmt = conn.prepare(
            r#"
            SELECT session_id, title, created_at, updated_at, message_count
            FROM conversations
            ORDER BY updated_at DESC
            "#,
        )?;

        let conversations = stmt.query_map([], |row| {
            Ok(ConversationMetadata {
                session_id: row.get(0)?,
                title: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
                message_count: row.get(4)?,
            })
        })?;

        let mut result = Vec::new();
        for conv in conversations {
            result.push(conv?);
        }

        Ok(result)
    }

    /// Delete a conversation and all its messages
    pub async fn delete_conversation(&self, session_id: &SessionId) -> Result<(), StorageError> {
        let conn = self.conn.lock().await;

        conn.execute(
            "DELETE FROM messages WHERE session_id = ?1",
            params![session_id.0],
        )?;

        conn.execute(
            "DELETE FROM conversations WHERE session_id = ?1",
            params![session_id.0],
        )?;

        Ok(())
    }

    /// Update conversation title
    pub async fn update_title(&self, session_id: &SessionId, title: &str) -> Result<(), StorageError> {
        let conn = self.conn.lock().await;

        conn.execute(
            "UPDATE conversations SET title = ?1 WHERE session_id = ?2",
            params![title, session_id.0],
        )?;

        Ok(())
    }

    /// Check if a conversation exists
    pub async fn conversation_exists(&self, session_id: &SessionId) -> Result<bool, StorageError> {
        let conn = self.conn.lock().await;

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM conversations WHERE session_id = ?1",
            params![session_id.0],
            |row| row.get(0),
        )?;

        Ok(count > 0)
    }

    /// Get current Unix timestamp in milliseconds
    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_creation() {
        // This test requires a real filesystem
        // Skip in CI environments without write access
        if std::env::var("CI").is_ok() {
            return;
        }

        let storage = ConversationStorage::new();
        assert!(storage.is_ok());
    }
}
