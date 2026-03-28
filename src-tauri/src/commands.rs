use crate::db::Database;
use crate::models::*;
use crate::player;
use crate::scanner;
use rand::seq::SliceRandom;
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
    description: Option<String>,
    is_smart: bool,
) -> Result<Playlist, String> {
    db.create_playlist(&name, description.as_deref(), is_smart)
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
pub fn add_to_playlist(
    db: DbState<'_>,
    playlist_id: String,
    media_ids: Vec<String>,
) -> Result<(), String> {
    db.add_to_playlist(&playlist_id, &media_ids)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_playlist_items(db: DbState<'_>, playlist_id: String) -> Result<Vec<MediaFile>, String> {
    db.get_playlist_items(&playlist_id)
        .map_err(|e| e.to_string())
}

// === Filter commands ===

#[tauri::command]
pub fn add_playlist_filter(
    db: DbState<'_>,
    playlist_id: String,
    filter_type: String,
    operator: String,
    value: String,
) -> Result<(), String> {
    let filter = PlaylistFilter {
        filter_type,
        operator,
        value,
    };
    db.add_playlist_filter(&playlist_id, &filter)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_playlist_filters(
    db: DbState<'_>,
    playlist_id: String,
) -> Result<Vec<PlaylistFilter>, String> {
    db.get_playlist_filters(&playlist_id)
        .map_err(|e| e.to_string())
}

// === Player commands ===

#[tauri::command]
pub fn play_file(path: String) -> Result<(), String> {
    player::play_file(&path)
}

#[tauri::command]
pub fn play_playlist(db: DbState<'_>, playlist_id: String) -> Result<(), String> {
    let items = db
        .get_playlist_items(&playlist_id)
        .map_err(|e| e.to_string())?;

    if items.is_empty() {
        return Err("Playlist is empty".to_string());
    }

    // Check playback mode
    let playlists = db.list_playlists().map_err(|e| e.to_string())?;
    let playlist = playlists
        .iter()
        .find(|p| p.id == playlist_id)
        .ok_or("Playlist not found")?;

    let mut paths: Vec<String> = items.iter().map(|f| f.path.clone()).collect();

    if playlist.playback_mode == "random" {
        let mut rng = rand::rng();
        paths.shuffle(&mut rng);
    }

    player::play_files(&paths)
}
