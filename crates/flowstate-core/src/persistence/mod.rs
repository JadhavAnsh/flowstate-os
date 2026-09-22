use std::path::Path;

use chrono::{DateTime, Utc};
use flowstate_protocol::Event;
use rusqlite::{params, Connection};

use crate::error::{CoreError, CoreResult};

mod models;

pub use models::{ConversationRow, MessageRow, ProviderConfigRow, RunRow, TaskRow};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &Path) -> CoreResult<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> CoreResult<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS conversations (
              id TEXT PRIMARY KEY,
              title TEXT NOT NULL,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS messages (
              id TEXT PRIMARY KEY,
              conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
              role TEXT NOT NULL,
              content TEXT NOT NULL,
              created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS events (
              id TEXT PRIMARY KEY,
              occurred_at TEXT NOT NULL,
              event_json TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS tasks (
              id TEXT PRIMARY KEY,
              title TEXT NOT NULL,
              status TEXT NOT NULL,
              conversation_id TEXT,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS runs (
              id TEXT PRIMARY KEY,
              task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
              status TEXT NOT NULL,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS provider_config (
              provider_id TEXT PRIMARY KEY,
              default_model TEXT NOT NULL,
              updated_at TEXT NOT NULL
            );
            ",
        )?;
        Ok(())
    }

    pub fn insert_event(&self, event: &Event) -> CoreResult<()> {
        let event_json = serde_json::to_string(event)?;
        self.conn.execute(
            "INSERT INTO events (id, occurred_at, event_json) VALUES (?1, ?2, ?3)",
            params![event.id.to_string(), event.occurred_at.to_rfc3339(), event_json],
        )?;
        Ok(())
    }

    pub fn list_events(&self, limit: usize) -> CoreResult<Vec<Event>> {
        let mut stmt = self
            .conn
            .prepare("SELECT event_json FROM events ORDER BY occurred_at ASC LIMIT ?1")?;
        let rows = stmt.query_map([limit as i64], |row| {
            let event_json: String = row.get(0)?;
            Ok(event_json)
        })?;
        rows.map(|row| {
            let event_json = row?;
            let event: Event = serde_json::from_str(&event_json)?;
            Ok(event)
        })
        .collect()
    }

    pub fn create_conversation(&self, title: &str) -> CoreResult<ConversationRow> {
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();
        self.conn.execute(
            "INSERT INTO conversations (id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            params![id, title, now.to_rfc3339(), now.to_rfc3339()],
        )?;
        Ok(ConversationRow {
            id,
            title: title.to_string(),
            created_at: now,
            updated_at: now,
        })
    }

    pub fn list_conversations(&self) -> CoreResult<Vec<ConversationRow>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, title, created_at, updated_at FROM conversations ORDER BY updated_at DESC")?;
        let rows = stmt.query_map([], |row| {
            Ok(ConversationRow {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at: parse_dt(row.get(2)?)?,
                updated_at: parse_dt(row.get(3)?)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(CoreError::from)
    }

    pub fn touch_conversation(&self, id: &str) -> CoreResult<()> {
        self.conn.execute(
            "UPDATE conversations SET updated_at = ?1 WHERE id = ?2",
            params![Utc::now().to_rfc3339(), id],
        )?;
        Ok(())
    }

    pub fn insert_message(&self, conversation_id: &str, role: &str, content: &str) -> CoreResult<MessageRow> {
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();
        self.conn.execute(
            "INSERT INTO messages (id, conversation_id, role, content, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, conversation_id, role, content, now.to_rfc3339()],
        )?;
        self.touch_conversation(conversation_id)?;
        Ok(MessageRow {
            id,
            conversation_id: conversation_id.to_string(),
            role: role.to_string(),
            content: content.to_string(),
            created_at: now,
        })
    }

    pub fn list_messages(&self, conversation_id: &str) -> CoreResult<Vec<MessageRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, conversation_id, role, content, created_at FROM messages
             WHERE conversation_id = ?1 ORDER BY created_at ASC",
        )?;
        let rows = stmt.query_map([conversation_id], |row| {
            Ok(MessageRow {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                created_at: parse_dt(row.get(4)?)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(CoreError::from)
    }

    pub fn insert_task(&self, title: &str, conversation_id: Option<&str>) -> CoreResult<TaskRow> {
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();
        self.conn.execute(
            "INSERT INTO tasks (id, title, status, conversation_id, created_at, updated_at)
             VALUES (?1, ?2, 'pending', ?3, ?4, ?5)",
            params![id, title, conversation_id, now.to_rfc3339(), now.to_rfc3339()],
        )?;
        Ok(TaskRow {
            id,
            title: title.to_string(),
            status: "pending".to_string(),
            conversation_id: conversation_id.map(str::to_string),
            created_at: now,
            updated_at: now,
        })
    }

    pub fn update_task_status(&self, task_id: &str, status: &str) -> CoreResult<()> {
        self.conn.execute(
            "UPDATE tasks SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status, Utc::now().to_rfc3339(), task_id],
        )?;
        Ok(())
    }

    pub fn update_run_status(&self, run_id: &str, status: &str) -> CoreResult<()> {
        self.conn.execute(
            "UPDATE runs SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status, Utc::now().to_rfc3339(), run_id],
        )?;
        Ok(())
    }

    pub fn get_task(&self, task_id: &str) -> CoreResult<Option<TaskRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, status, conversation_id, created_at, updated_at FROM tasks WHERE id = ?1",
        )?;
        let mut rows = stmt.query([task_id])?;
        if let Some(row) = rows.next()? {
            return Ok(Some(TaskRow {
                id: row.get(0)?,
                title: row.get(1)?,
                status: row.get(2)?,
                conversation_id: row.get(3)?,
                created_at: parse_dt(row.get(4)?)?,
                updated_at: parse_dt(row.get(5)?)?,
            }));
        }
        Ok(None)
    }

    pub fn get_run(&self, run_id: &str) -> CoreResult<Option<RunRow>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, task_id, status, created_at, updated_at FROM runs WHERE id = ?1")?;
        let mut rows = stmt.query([run_id])?;
        if let Some(row) = rows.next()? {
            return Ok(Some(RunRow {
                id: row.get(0)?,
                task_id: row.get(1)?,
                status: row.get(2)?,
                created_at: parse_dt(row.get(3)?)?,
                updated_at: parse_dt(row.get(4)?)?,
            }));
        }
        Ok(None)
    }

    pub fn list_tasks_for_conversation(&self, conversation_id: &str, limit: usize) -> CoreResult<Vec<TaskRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, status, conversation_id, created_at, updated_at FROM tasks
             WHERE conversation_id = ?1 ORDER BY created_at DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![conversation_id, limit as i64], |row| {
            Ok(TaskRow {
                id: row.get(0)?,
                title: row.get(1)?,
                status: row.get(2)?,
                conversation_id: row.get(3)?,
                created_at: parse_dt(row.get(4)?)?,
                updated_at: parse_dt(row.get(5)?)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(CoreError::from)
    }

    pub fn insert_run(&self, task_id: &str) -> CoreResult<RunRow> {
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();
        self.conn.execute(
            "INSERT INTO runs (id, task_id, status, created_at, updated_at) VALUES (?1, ?2, 'running', ?3, ?4)",
            params![id, task_id, now.to_rfc3339(), now.to_rfc3339()],
        )?;
        Ok(RunRow {
            id,
            task_id: task_id.to_string(),
            status: "running".to_string(),
            created_at: now,
            updated_at: now,
        })
    }

    pub fn get_provider_config(&self, provider_id: &str) -> CoreResult<Option<ProviderConfigRow>> {
        let mut stmt = self
            .conn
            .prepare("SELECT provider_id, default_model, updated_at FROM provider_config WHERE provider_id = ?1")?;
        let mut rows = stmt.query([provider_id])?;
        if let Some(row) = rows.next()? {
            return Ok(Some(ProviderConfigRow {
                provider_id: row.get(0)?,
                default_model: row.get(1)?,
                updated_at: parse_dt(row.get(2)?)?,
            }));
        }
        Ok(None)
    }

    pub fn upsert_provider_config(&self, provider_id: &str, default_model: &str) -> CoreResult<ProviderConfigRow> {
        let now = Utc::now();
        self.conn.execute(
            "INSERT INTO provider_config (provider_id, default_model, updated_at)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(provider_id) DO UPDATE SET default_model = excluded.default_model, updated_at = excluded.updated_at",
            params![provider_id, default_model, now.to_rfc3339()],
        )?;
        Ok(ProviderConfigRow {
            provider_id: provider_id.to_string(),
            default_model: default_model.to_string(),
            updated_at: now,
        })
    }
}

fn parse_dt(raw: String) -> rusqlite::Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(&raw)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))
}
