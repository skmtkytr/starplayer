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

            CREATE TABLE IF NOT EXISTS playlists (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                playback_mode TEXT NOT NULL DEFAULT 'sequential',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS playlist_filters (
                id TEXT PRIMARY KEY,
                playlist_id TEXT NOT NULL,
                field TEXT NOT NULL,
                operator TEXT NOT NULL,
                value TEXT NOT NULL,
                FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_media_workspace ON media_files(workspace_id);
            CREATE INDEX IF NOT EXISTS idx_media_filename ON media_files(filename);
            CREATE INDEX IF NOT EXISTS idx_media_series ON media_files(series_name, series_number);
            ",
        )?;
        Ok(())
    }

    pub fn conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap()
    }
}

use crate::models::*;

impl Database {
    // === Workspace operations ===

    pub fn add_workspace(&self, name: &str, path: &str) -> Result<Workspace> {
        let conn = self.conn();
        let existing: Option<Workspace> = conn
            .query_row(
                "SELECT id, name, path FROM workspaces WHERE path = ?1",
                params![path],
                |row| {
                    Ok(Workspace {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        path: row.get(2)?,
                    })
                },
            )
            .ok();
        if let Some(ws) = existing {
            if ws.name != name {
                conn.execute(
                    "UPDATE workspaces SET name = ?1, updated_at = datetime('now') WHERE id = ?2",
                    params![name, ws.id],
                )?;
                return Ok(Workspace {
                    id: ws.id,
                    name: name.to_string(),
                    path: ws.path,
                });
            }
            return Ok(ws);
        }
        let id = uuid::Uuid::new_v4().to_string();
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
                file.id, file.workspace_id, file.path, file.filename,
                file.extension, file.size_bytes, file.series_name, file.series_number,
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
        let rows = stmt.query_map(params_ref.as_slice(), Self::row_to_media_file)?;
        rows.collect()
    }

    /// Query media files matching a playlist's filter conditions.
    pub fn query_playlist_files(&self, playlist_id: &str) -> Result<Vec<MediaFile>> {
        let filters = self.get_playlist_filters(playlist_id)?;

        let conn = self.conn();
        let mut sql = String::from(
            "SELECT id, workspace_id, path, filename, extension, size_bytes,
                    duration_secs, width, height, series_name, series_number
             FROM media_files WHERE 1=1",
        );
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        for f in &filters {
            let col = match f.field.as_str() {
                "filename" => "filename",
                "extension" => "extension",
                "path" => "path",
                "series_name" => "series_name",
                "workspace_id" => "workspace_id",
                _ => continue,
            };

            match f.operator.as_str() {
                "contains" => {
                    sql.push_str(&format!(" AND {col} LIKE ?"));
                    param_values.push(Box::new(format!("%{}%", f.value)));
                }
                "not_contains" => {
                    sql.push_str(&format!(" AND ({col} NOT LIKE ? OR {col} IS NULL)"));
                    param_values.push(Box::new(format!("%{}%", f.value)));
                }
                "equals" => {
                    sql.push_str(&format!(" AND {col} = ?"));
                    param_values.push(Box::new(f.value.clone()));
                }
                "starts_with" => {
                    sql.push_str(&format!(" AND {col} LIKE ?"));
                    param_values.push(Box::new(format!("{}%", f.value)));
                }
                "ends_with" => {
                    sql.push_str(&format!(" AND {col} LIKE ?"));
                    param_values.push(Box::new(format!("%{}", f.value)));
                }
                _ => continue,
            }
        }
        sql.push_str(" ORDER BY series_name, series_number, filename");

        let mut stmt = conn.prepare(&sql)?;
        let params_ref: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        let rows = stmt.query_map(params_ref.as_slice(), Self::row_to_media_file)?;
        rows.collect()
    }

    fn row_to_media_file(row: &rusqlite::Row<'_>) -> rusqlite::Result<MediaFile> {
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
    }

    pub fn remove_media_file_by_path(&self, path: &str) -> Result<usize> {
        let conn = self.conn();
        let affected = conn.execute("DELETE FROM media_files WHERE path = ?1", params![path])?;
        Ok(affected)
    }

    pub fn list_workspace_media_paths(&self, workspace_id: &str) -> Result<Vec<String>> {
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT path FROM media_files WHERE workspace_id = ?1")?;
        let rows = stmt.query_map(params![workspace_id], |row| row.get::<_, String>(0))?;
        rows.collect()
    }

    pub fn get_media_file(&self, id: &str) -> Result<Option<MediaFile>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT id, workspace_id, path, filename, extension, size_bytes,
                    duration_secs, width, height, series_name, series_number
             FROM media_files WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], Self::row_to_media_file)?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    // === Playlist operations ===

    pub fn create_playlist(&self, name: &str, filters: &[PlaylistFilter]) -> Result<Playlist> {
        let id = uuid::Uuid::new_v4().to_string();
        let conn = self.conn();
        conn.execute(
            "INSERT INTO playlists (id, name) VALUES (?1, ?2)",
            params![id, name],
        )?;
        Self::insert_filters(&conn, &id, filters)?;
        Ok(Playlist {
            id,
            name: name.to_string(),
            playback_mode: "sequential".to_string(),
            filters: filters.to_vec(),
        })
    }

    pub fn list_playlists(&self) -> Result<Vec<Playlist>> {
        let conn = self.conn();
        let mut stmt =
            conn.prepare("SELECT id, name, playback_mode FROM playlists ORDER BY name")?;
        let playlists: Vec<(String, String, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
            .collect::<Result<Vec<_>>>()?;

        let mut result = Vec::new();
        for (id, name, mode) in playlists {
            let mut fstmt = conn.prepare(
                "SELECT field, operator, value FROM playlist_filters WHERE playlist_id = ?1",
            )?;
            let filters: Vec<PlaylistFilter> = fstmt
                .query_map(params![id], |row| {
                    Ok(PlaylistFilter {
                        field: row.get(0)?,
                        operator: row.get(1)?,
                        value: row.get(2)?,
                    })
                })?
                .collect::<Result<Vec<_>>>()?;
            result.push(Playlist {
                id,
                name,
                playback_mode: mode,
                filters,
            });
        }
        Ok(result)
    }

    pub fn update_playlist(&self, id: &str, name: &str, filters: &[PlaylistFilter]) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "UPDATE playlists SET name = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![name, id],
        )?;
        conn.execute(
            "DELETE FROM playlist_filters WHERE playlist_id = ?1",
            params![id],
        )?;
        Self::insert_filters(&conn, id, filters)?;
        Ok(())
    }

    fn insert_filters(
        conn: &Connection,
        playlist_id: &str,
        filters: &[PlaylistFilter],
    ) -> Result<()> {
        for f in filters {
            conn.execute(
                "INSERT INTO playlist_filters (id, playlist_id, field, operator, value)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    uuid::Uuid::new_v4().to_string(),
                    playlist_id,
                    f.field,
                    f.operator,
                    f.value
                ],
            )?;
        }
        Ok(())
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

    fn get_playlist_filters(&self, playlist_id: &str) -> Result<Vec<PlaylistFilter>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT field, operator, value FROM playlist_filters WHERE playlist_id = ?1",
        )?;
        let rows = stmt.query_map(params![playlist_id], |row| {
            Ok(PlaylistFilter {
                field: row.get(0)?,
                operator: row.get(1)?,
                value: row.get(2)?,
            })
        })?;
        rows.collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_test_db() -> Database {
        let tmp = std::env::temp_dir().join(format!("starplayer-test-{}", uuid::Uuid::new_v4()));
        Database::new(tmp).expect("db init")
    }

    fn sample_insert(workspace_id: &str, path: &str) -> MediaFileInsert {
        MediaFileInsert {
            id: uuid::Uuid::new_v4().to_string(),
            workspace_id: workspace_id.to_string(),
            path: path.to_string(),
            filename: path.rsplit('/').next().unwrap_or(path).to_string(),
            extension: "mp4".to_string(),
            size_bytes: 123,
            series_name: None,
            series_number: None,
        }
    }

    #[test]
    fn remove_media_file_by_path_deletes_matching_row() {
        let db = new_test_db();
        let ws = db.add_workspace("w", "/tmp/w").unwrap();
        db.upsert_media_file(&sample_insert(&ws.id, "/tmp/w/a.mp4"))
            .unwrap();
        db.upsert_media_file(&sample_insert(&ws.id, "/tmp/w/b.mp4"))
            .unwrap();

        let affected = db.remove_media_file_by_path("/tmp/w/a.mp4").unwrap();
        assert_eq!(affected, 1);

        let remaining = db.list_workspace_media_paths(&ws.id).unwrap();
        assert_eq!(remaining, vec!["/tmp/w/b.mp4".to_string()]);
    }

    #[test]
    fn remove_media_file_by_path_returns_zero_when_missing() {
        let db = new_test_db();
        let affected = db.remove_media_file_by_path("/nonexistent").unwrap();
        assert_eq!(affected, 0);
    }

    #[test]
    fn list_workspace_media_paths_scopes_to_workspace() {
        let db = new_test_db();
        let ws1 = db.add_workspace("w1", "/tmp/w1").unwrap();
        let ws2 = db.add_workspace("w2", "/tmp/w2").unwrap();
        db.upsert_media_file(&sample_insert(&ws1.id, "/tmp/w1/x.mp4"))
            .unwrap();
        db.upsert_media_file(&sample_insert(&ws2.id, "/tmp/w2/y.mp4"))
            .unwrap();

        let paths = db.list_workspace_media_paths(&ws1.id).unwrap();
        assert_eq!(paths, vec!["/tmp/w1/x.mp4".to_string()]);
    }
}
