use crate::db::Database;
use crate::models::*;
use crate::scanner;
use tauri::State;

type DbState<'a> = State<'a, Database>;

// === Workspace commands ===

#[tauri::command]
pub fn add_workspace(db: DbState<'_>, name: String, path: String) -> Result<Workspace, String> {
    db.add_workspace(&name, &path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_workspaces(db: DbState<'_>) -> Result<Vec<Workspace>, String> {
    db.list_workspaces().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_workspace(db: DbState<'_>, id: String) -> Result<(), String> {
    db.remove_workspace(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn scan_workspace(
    db: DbState<'_>,
    workspace_id: String,
    path: String,
) -> Result<ScanResult, String> {
    scanner::scan_workspace(&db, &workspace_id, &path)
}

// === Media commands ===

#[tauri::command]
pub fn list_media_files(
    db: DbState<'_>,
    workspace_id: Option<String>,
    search: Option<String>,
    extension: Option<String>,
) -> Result<Vec<MediaFile>, String> {
    db.list_media_files(
        workspace_id.as_deref(),
        search.as_deref(),
        extension.as_deref(),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_media_file(db: DbState<'_>, id: String) -> Result<Option<MediaFile>, String> {
    db.get_media_file(&id).map_err(|e| e.to_string())
}

// === Playlist commands ===

#[tauri::command]
pub fn create_playlist(
    db: DbState<'_>,
    name: String,
    filters: Vec<PlaylistFilter>,
) -> Result<Playlist, String> {
    db.create_playlist(&name, &filters)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_playlist(
    db: DbState<'_>,
    id: String,
    name: String,
    filters: Vec<PlaylistFilter>,
) -> Result<(), String> {
    db.update_playlist(&id, &name, &filters)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_playlists(db: DbState<'_>) -> Result<Vec<Playlist>, String> {
    db.list_playlists().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_playlist(db: DbState<'_>, id: String) -> Result<(), String> {
    db.delete_playlist(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_playback_mode(db: DbState<'_>, playlist_id: String, mode: String) -> Result<(), String> {
    let valid_modes = ["sequential", "random", "repeat"];
    if !valid_modes.contains(&mode.as_str()) {
        return Err(format!(
            "Invalid playback mode: {mode}. Valid: {valid_modes:?}"
        ));
    }
    db.set_playback_mode(&playlist_id, &mode)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_playlist_files(db: DbState<'_>, playlist_id: String) -> Result<Vec<MediaFile>, String> {
    db.query_playlist_files(&playlist_id)
        .map_err(|e| e.to_string())
}
