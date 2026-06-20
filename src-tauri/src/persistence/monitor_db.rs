use std::{path::PathBuf, sync::{Arc, Mutex}};

use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection};

use crate::models::{MonitorEntry, MonitorLevel};

#[derive(Clone)]
pub struct MonitorDb {
    connection: Arc<Mutex<Connection>>,
}

impl MonitorDb {
    pub fn new(path: PathBuf) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("failed creating monitor DB directory {}", parent.display())
            })?;
        }

        let connection = Connection::open(&path)
            .with_context(|| format!("failed opening monitor DB at {}", path.display()))?;
        let db = Self {
            connection: Arc::new(Mutex::new(connection)),
        };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self
            .connection
            .lock()
            .map_err(|_| anyhow::anyhow!("monitor DB mutex poisoned"))?;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS monitor_entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                level TEXT NOT NULL,
                category TEXT NOT NULL,
                message TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_monitor_entries_timestamp
                ON monitor_entries(timestamp DESC);
            "#,
        )
        .context("failed creating monitor DB schema")?;
        Ok(())
    }

    const MAX_ENTRIES: i64 = 1000;

    pub async fn append(&self, level: MonitorLevel, category: &str, message: &str) -> Result<()> {
        let db = self.clone();
        let level = level.clone();
        let category = category.to_string();
        let message = message.to_string();
        tokio::task::spawn_blocking(move || db.append_sync(level, &category, &message))
            .await?
    }

    fn append_sync(&self, level: MonitorLevel, category: &str, message: &str) -> Result<()> {
        let timestamp = Utc::now().to_rfc3339();
        let conn = self
            .connection
            .lock()
            .map_err(|_| anyhow::anyhow!("monitor DB mutex poisoned"))?;

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM monitor_entries", [], |row| row.get(0))
            .unwrap_or(0);
        if count >= Self::MAX_ENTRIES {
            conn.execute(
                "DELETE FROM monitor_entries WHERE id NOT IN (SELECT id FROM monitor_entries ORDER BY id DESC LIMIT ?1)",
                params![Self::MAX_ENTRIES],
            )
            .ok();
        }

        conn.execute(
            "INSERT INTO monitor_entries(timestamp, level, category, message) VALUES(?1, ?2, ?3, ?4)",
            params![timestamp, level.as_str(), category, message],
        )
        .context("failed inserting monitor entry")?;
        Ok(())
    }

    pub async fn list_recent(&self, limit: usize) -> Result<Vec<MonitorEntry>> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || db.list_recent_sync(limit)).await?
    }

    fn list_recent_sync(&self, limit: usize) -> Result<Vec<MonitorEntry>> {
        let conn = self
            .connection
            .lock()
            .map_err(|_| anyhow::anyhow!("monitor DB mutex poisoned"))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, timestamp, level, category, message
                 FROM monitor_entries
                 ORDER BY id DESC
                 LIMIT ?1",
            )
            .context("failed preparing monitor entry query")?;

        let rows = stmt
            .query_map(params![limit as i64], |row| {
                Ok(MonitorEntry {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    level: MonitorLevel::from_db(&row.get::<_, String>(2)?),
                    category: row.get(3)?,
                    message: row.get(4)?,
                })
            })
            .context("failed querying monitor entries")?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(row.context("failed decoding monitor row")?);
        }

        Ok(entries)
    }
}

