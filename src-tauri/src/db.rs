use rusqlite::{params, Connection, Result};
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(app_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&app_dir).ok();
        let db_path = app_dir.join("starplayer.db");
        let conn = Connection::open(db_path)?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.initialize()?;
        Ok(db)
    }

    fn initialize(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS workspaces (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                path TEXT NOT NULL UNIQUE,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS media_files (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL,
                path TEXT NOT NULL UNIQUE,
                filename TEXT NOT NULL,
                extension TEXT NOT NULL,
                size_bytes INTEGER NOT NULL DEFAULT 0,
                duration_secs REAL,
                width INTEGER,
                height INTEGER,
                series_name TEXT,
                series_number INTEGER,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS tags (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE
            );

            CREATE TABLE IF NOT EXISTS media_tags (
                media_id TEXT NOT NULL,
                tag_id TEXT NOT NULL,
                PRIMARY KEY (media_id, tag_id),
                FOREIGN KEY (media_id) REFERENCES media_files(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS playlists (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                is_smart INTEGER NOT NULL DEFAULT 0,
                playback_mode TEXT NOT NULL DEFAULT 'sequential',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS playlist_items (
                id TEXT PRIMARY KEY,
                playlist_id TEXT NOT NULL,
                media_id TEXT NOT NULL,
                position INTEGER NOT NULL,
                FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE,
                FOREIGN KEY (media_id) REFERENCES media_files(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS playlist_filters (
                id TEXT PRIMARY KEY,
                playlist_id TEXT NOT NULL,
                filter_type TEXT NOT NULL,
                operator TEXT NOT NULL,
                value TEXT NOT NULL,
                FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_media_workspace ON media_files(workspace_id);
            CREATE INDEX IF NOT EXISTS idx_media_filename ON media_files(filename);
            CREATE INDEX IF NOT EXISTS idx_media_series ON media_files(series_name, series_number);
            CREATE INDEX IF NOT EXISTS idx_playlist_items_playlist ON playlist_items(playlist_id, position);
            ",
        )?;
        Ok(())
    }

    pub fn conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap()
    }
}

// === Workspace operations ===
use crate::models::*;

impl Database {
    pub fn add_workspace(&self, name: &str, path: &str) -> Result<Workspace> {
        let id = uuid::Uuid::new_v4().to_string();
        let conn = self.conn();
        conn.execute(
            "INSERT INTO workspaces (id, name, path) VALUES (?1, ?2, ?3)",
            params![id, name, path],
        )?;
        Ok(Workspace {
            id,
            name: name.to_string(),
            path: path.to_string(),
        })
    }

    pub fn list_workspaces(&self) -> Result<Vec<Workspace>> {
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT id, name, path FROM workspaces ORDER BY name")?;
        let rows = stmt.query_map([], |row| {
            Ok(Workspace {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
            })
        })?;
        rows.collect()
    }

    pub fn remove_workspace(&self, id: &str) -> Result<()> {
        let conn = self.conn();
        conn.execute("DELETE FROM workspaces WHERE id = ?1", params![id])?;
        Ok(())
    }

    // === Media file operations ===

    pub fn upsert_media_file(&self, file: &MediaFileInsert) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO media_files (id, workspace_id, path, filename, extension, size_bytes, series_name, series_number)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(path) DO UPDATE SET
                filename = excluded.filename,
                size_bytes = excluded.size_bytes,
                series_name = excluded.series_name,
                series_number = excluded.series_number,
                updated_at = datetime('now')",
            params![
                file.id,
                file.workspace_id,
                file.path,
                file.filename,
                file.extension,
                file.size_bytes,
                file.series_name,
                file.series_number,
            ],
        )?;
        Ok(())
    }

    pub fn list_media_files(
        &self,
        workspace_id: Option<&str>,
        search: Option<&str>,
        extension: Option<&str>,
    ) -> Result<Vec<MediaFile>> {
        let conn = self.conn();
        let mut sql = String::from(
            "SELECT id, workspace_id, path, filename, extension, size_bytes,
                    duration_secs, width, height, series_name, series_number
             FROM media_files WHERE 1=1",
        );
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(wid) = workspace_id {
            sql.push_str(" AND workspace_id = ?");
            param_values.push(Box::new(wid.to_string()));
        }
        if let Some(s) = search {
            sql.push_str(" AND filename LIKE ?");
            param_values.push(Box::new(format!("%{s}%")));
        }
        if let Some(ext) = extension {
            sql.push_str(" AND extension = ?");
            param_values.push(Box::new(ext.to_string()));
        }
        sql.push_str(" ORDER BY filename");

        let mut stmt = conn.prepare(&sql)?;
        let params_ref: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        let rows = stmt.query_map(params_ref.as_slice(), |row| {
            Ok(MediaFile {
                id: row.get(0)?,
                workspace_id: row.get(1)?,
                path: row.get(2)?,
                filename: row.get(3)?,
                extension: row.get(4)?,
                size_bytes: row.get(5)?,
                duration_secs: row.get(6)?,
                width: row.get(7)?,
                height: row.get(8)?,
                series_name: row.get(9)?,
                series_number: row.get(10)?,
            })
        })?;
        rows.collect()
    }

    pub fn get_media_file(&self, id: &str) -> Result<Option<MediaFile>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT id, workspace_id, path, filename, extension, size_bytes,
                    duration_secs, width, height, series_name, series_number
             FROM media_files WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(MediaFile {
                id: row.get(0)?,
                workspace_id: row.get(1)?,
                path: row.get(2)?,
                filename: row.get(3)?,
                extension: row.get(4)?,
                size_bytes: row.get(5)?,
                duration_secs: row.get(6)?,
                width: row.get(7)?,
                height: row.get(8)?,
                series_name: row.get(9)?,
                series_number: row.get(10)?,
            })
        })?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    // === Playlist operations ===

    pub fn create_playlist(
        &self,
        name: &str,
        description: Option<&str>,
        is_smart: bool,
    ) -> Result<Playlist> {
        let id = uuid::Uuid::new_v4().to_string();
        let conn = self.conn();
        conn.execute(
            "INSERT INTO playlists (id, name, description, is_smart) VALUES (?1, ?2, ?3, ?4)",
            params![id, name, description, is_smart as i32],
        )?;
        Ok(Playlist {
            id,
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            is_smart,
            playback_mode: "sequential".to_string(),
            item_count: 0,
        })
    }

    pub fn list_playlists(&self) -> Result<Vec<Playlist>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT p.id, p.name, p.description, p.is_smart, p.playback_mode,
                    (SELECT COUNT(*) FROM playlist_items pi WHERE pi.playlist_id = p.id) as item_count
             FROM playlists p ORDER BY p.name",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Playlist {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                is_smart: row.get::<_, i32>(3)? != 0,
                playback_mode: row.get(4)?,
                item_count: row.get(5)?,
            })
        })?;
        rows.collect()
    }

    pub fn delete_playlist(&self, id: &str) -> Result<()> {
        let conn = self.conn();
        conn.execute("DELETE FROM playlists WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn set_playback_mode(&self, playlist_id: &str, mode: &str) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "UPDATE playlists SET playback_mode = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![mode, playlist_id],
        )?;
        Ok(())
    }

    pub fn add_to_playlist(&self, playlist_id: &str, media_ids: &[String]) -> Result<()> {
        let conn = self.conn();
        let max_pos: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(position), -1) FROM playlist_items WHERE playlist_id = ?1",
                params![playlist_id],
                |row| row.get(0),
            )
            .unwrap_or(-1);

        for (i, media_id) in media_ids.iter().enumerate() {
            let id = uuid::Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO playlist_items (id, playlist_id, media_id, position) VALUES (?1, ?2, ?3, ?4)",
                params![id, playlist_id, media_id, max_pos + 1 + i as i64],
            )?;
        }
        Ok(())
    }

    pub fn get_playlist_items(&self, playlist_id: &str) -> Result<Vec<MediaFile>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT m.id, m.workspace_id, m.path, m.filename, m.extension, m.size_bytes,
                    m.duration_secs, m.width, m.height, m.series_name, m.series_number
             FROM playlist_items pi
             JOIN media_files m ON m.id = pi.media_id
             WHERE pi.playlist_id = ?1
             ORDER BY pi.position",
        )?;
        let rows = stmt.query_map(params![playlist_id], |row| {
            Ok(MediaFile {
                id: row.get(0)?,
                workspace_id: row.get(1)?,
                path: row.get(2)?,
                filename: row.get(3)?,
                extension: row.get(4)?,
                size_bytes: row.get(5)?,
                duration_secs: row.get(6)?,
                width: row.get(7)?,
                height: row.get(8)?,
                series_name: row.get(9)?,
                series_number: row.get(10)?,
            })
        })?;
        rows.collect()
    }

    // === Playlist filter operations ===

    pub fn add_playlist_filter(&self, playlist_id: &str, filter: &PlaylistFilter) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO playlist_filters (id, playlist_id, filter_type, operator, value)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                uuid::Uuid::new_v4().to_string(),
                playlist_id,
                filter.filter_type,
                filter.operator,
                filter.value,
            ],
        )?;
        Ok(())
    }

    pub fn get_playlist_filters(&self, playlist_id: &str) -> Result<Vec<PlaylistFilter>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT filter_type, operator, value FROM playlist_filters WHERE playlist_id = ?1",
        )?;
        let rows = stmt.query_map(params![playlist_id], |row| {
            Ok(PlaylistFilter {
                filter_type: row.get(0)?,
                operator: row.get(1)?,
                value: row.get(2)?,
            })
        })?;
        rows.collect()
    }
}
