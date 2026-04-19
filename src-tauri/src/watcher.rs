use crate::db::Database;
use crate::scanner;
use notify::{Config, Event, PollWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

pub const MEDIA_CHANGED_EVENT: &str = "media-files-changed";

/// Polling interval. Network mounts (NFS/SMB/sshfs/...) do not deliver
/// inotify events, so we use a polling watcher that works on any filesystem
/// at the cost of detection latency.
const POLL_INTERVAL: Duration = Duration::from_secs(10);

pub struct WatcherRegistry {
    inner: Mutex<HashMap<String, PollWatcher>>,
}

impl WatcherRegistry {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }

    /// Start watching `path` recursively. Any previous watcher for the same
    /// workspace_id is replaced.
    pub fn watch(
        &self,
        workspace_id: String,
        path: String,
        app: AppHandle,
    ) -> Result<(), String> {
        let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
        let config = Config::default()
            .with_poll_interval(POLL_INTERVAL)
            .with_compare_contents(false);
        let mut watcher = PollWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            config,
        )
        .map_err(|e| format!("Failed to create watcher: {e}"))?;

        watcher
            .watch(Path::new(&path), RecursiveMode::Recursive)
            .map_err(|e| format!("Failed to watch {path}: {e}"))?;
        eprintln!(
            "[watcher] started (polling every {}s) workspace={workspace_id} path={path}",
            POLL_INTERVAL.as_secs()
        );

        let ws_id = workspace_id.clone();
        let app_clone = app.clone();
        thread::spawn(move || {
            for res in rx {
                match res {
                    Ok(event) => {
                        eprintln!(
                            "[watcher] {ws_id} event {:?} paths={:?}",
                            event.kind, event.paths
                        );
                        handle_event(&event, &ws_id, &app_clone);
                    }
                    Err(e) => eprintln!("[watcher] {ws_id} error: {e}"),
                }
            }
            eprintln!("[watcher] {ws_id} channel closed");
        });

        let mut map = self.inner.lock().unwrap();
        map.insert(workspace_id, watcher);
        Ok(())
    }

    pub fn unwatch(&self, workspace_id: &str) {
        let mut map = self.inner.lock().unwrap();
        map.remove(workspace_id);
    }
}

impl Default for WatcherRegistry {
    fn default() -> Self {
        Self::new()
    }
}

fn handle_event(event: &Event, workspace_id: &str, app: &AppHandle) {
    let db = app.state::<Database>();
    if process_event(event, workspace_id, &db) {
        eprintln!("[watcher] {workspace_id} emitting {MEDIA_CHANGED_EVENT}");
        if let Err(e) = app.emit(MEDIA_CHANGED_EVENT, workspace_id) {
            eprintln!("[watcher] {workspace_id} emit failed: {e}");
        }
    }
}

/// Apply a filesystem event to the database. Returns true if any row changed.
/// Split out from `handle_event` so it can be tested without a Tauri AppHandle.
pub(crate) fn process_event(event: &Event, workspace_id: &str, db: &Database) -> bool {
    let mut changed = false;

    for path in &event.paths {
        let path_str = path.to_string_lossy().to_string();

        if path.is_file() {
            if scanner::is_video_file(path)
                && let Some(insert) = scanner::build_insert(workspace_id, path)
                && db.upsert_media_file(&insert).is_ok()
            {
                changed = true;
            }
        } else if !path.exists() {
            // File removed or moved away. May be a file path or a directory
            // path — try the file delete first, then prune any descendants.
            if db
                .remove_media_file_by_path(&path_str)
                .map(|n| n > 0)
                .unwrap_or(false)
            {
                changed = true;
            }
            let prefix = if path_str.ends_with(std::path::MAIN_SEPARATOR) {
                path_str.clone()
            } else {
                format!("{path_str}{}", std::path::MAIN_SEPARATOR)
            };
            if let Ok(known) = db.list_workspace_media_paths(workspace_id) {
                for p in known {
                    if p.starts_with(&prefix)
                        && db
                            .remove_media_file_by_path(&p)
                            .map(|n| n > 0)
                            .unwrap_or(false)
                    {
                        changed = true;
                    }
                }
            }
        }
    }

    changed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use std::sync::mpsc;
    use std::time::Instant;

    fn poll_watcher(
        tx: mpsc::Sender<notify::Result<Event>>,
        interval: Duration,
    ) -> PollWatcher {
        let config = Config::default()
            .with_poll_interval(interval)
            .with_compare_contents(false);
        PollWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            config,
        )
        .unwrap()
    }

    fn drain_until<F: Fn(&Database) -> bool>(
        rx: &mpsc::Receiver<notify::Result<Event>>,
        db: &Database,
        ws_id: &str,
        timeout: Duration,
        pred: F,
    ) -> bool {
        let deadline = Instant::now() + timeout;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return pred(db);
            }
            match rx.recv_timeout(remaining) {
                Ok(Ok(event)) => {
                    process_event(&event, ws_id, db);
                    if pred(db) {
                        return true;
                    }
                }
                Ok(Err(_)) | Err(_) => return pred(db),
            }
        }
    }

    /// End-to-end: start a real PollWatcher on a temp dir, drop a video file
    /// in, drain events through `process_event`, and assert the DB row appears.
    #[test]
    fn watcher_detects_new_video_file() {
        let tmp = std::env::temp_dir().join(format!("starplayer-watcher-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&tmp).unwrap();
        let db = Database::new(tmp.join("_db")).unwrap();
        let ws = db.add_workspace("t", tmp.to_str().unwrap()).unwrap();

        let (tx, rx) = mpsc::channel();
        let mut watcher = poll_watcher(tx, Duration::from_millis(200));
        watcher.watch(&tmp, RecursiveMode::Recursive).unwrap();

        let new_file = tmp.join("new.mp4");
        std::fs::write(&new_file, b"x").unwrap();
        let new_path = new_file.to_string_lossy().to_string();

        let ok = drain_until(&rx, &db, &ws.id, Duration::from_secs(3), |db| {
            db.list_workspace_media_paths(&ws.id)
                .unwrap()
                .contains(&new_path)
        });

        std::fs::remove_dir_all(&tmp).ok();
        assert!(ok, "PollWatcher did not record the new file within 3s");
    }

    #[test]
    fn watcher_removes_deleted_video_file() {
        let tmp =
            std::env::temp_dir().join(format!("starplayer-watcher-rm-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&tmp).unwrap();
        let db = Database::new(tmp.join("_db")).unwrap();
        let ws = db.add_workspace("t", tmp.to_str().unwrap()).unwrap();

        let target = tmp.join("gone.mp4");
        std::fs::write(&target, b"x").unwrap();
        let target_path = target.to_string_lossy().to_string();
        db.upsert_media_file(&scanner::build_insert(&ws.id, &target).unwrap())
            .unwrap();

        let (tx, rx) = mpsc::channel();
        let mut watcher = poll_watcher(tx, Duration::from_millis(200));
        watcher.watch(&tmp, RecursiveMode::Recursive).unwrap();

        std::fs::remove_file(&target).unwrap();

        let ok = drain_until(&rx, &db, &ws.id, Duration::from_secs(3), |db| {
            !db.list_workspace_media_paths(&ws.id)
                .unwrap()
                .contains(&target_path)
        });

        std::fs::remove_dir_all(&tmp).ok();
        assert!(ok, "PollWatcher did not remove the deleted file within 3s");
    }

    #[test]
    fn watcher_detects_file_in_new_subdirectory() {
        let tmp =
            std::env::temp_dir().join(format!("starplayer-watcher-sub-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&tmp).unwrap();
        let db = Database::new(tmp.join("_db")).unwrap();
        let ws = db.add_workspace("t", tmp.to_str().unwrap()).unwrap();

        let (tx, rx) = mpsc::channel();
        let mut watcher = poll_watcher(tx, Duration::from_millis(200));
        watcher.watch(&tmp, RecursiveMode::Recursive).unwrap();

        let sub = tmp.join("nested");
        std::fs::create_dir(&sub).unwrap();
        let new_file = sub.join("show.mkv");
        std::fs::write(&new_file, b"x").unwrap();
        let new_path = new_file.to_string_lossy().to_string();

        let ok = drain_until(&rx, &db, &ws.id, Duration::from_secs(4), |db| {
            db.list_workspace_media_paths(&ws.id)
                .unwrap()
                .contains(&new_path)
        });

        std::fs::remove_dir_all(&tmp).ok();
        assert!(
            ok,
            "PollWatcher did not pick up the file in a new subdirectory within 4s"
        );
    }
}
