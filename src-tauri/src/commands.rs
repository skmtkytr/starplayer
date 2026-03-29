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

// === Player commands ===

fn find_mpv() -> Result<std::path::PathBuf, String> {
    // 1. Check bundled mpv next to the executable
    if let Ok(exe) = std::env::current_exe() {
        let exe_dir = exe.parent().unwrap_or(std::path::Path::new("."));

        // macOS: binaries/mpv.app/Contents/MacOS/mpv (dev) or ../Resources/mpv.app/... (bundle)
        for candidate in [
            exe_dir.join("binaries/mpv.app/Contents/MacOS/mpv"),
            exe_dir.join("mpv.app/Contents/MacOS/mpv"),
            exe_dir.join("../Resources/mpv.app/Contents/MacOS/mpv"),
            // Windows: mpv.exe next to starplayer.exe
            exe_dir.join("mpv.exe"),
            exe_dir.join("binaries/mpv.exe"),
            // Dev mode: src-tauri/binaries/
            exe_dir.join("../../binaries/mpv.app/Contents/MacOS/mpv"),
            exe_dir.join("../../binaries/mpv.exe"),
        ] {
            if candidate.exists() {
                return Ok(candidate);
            }
        }
    }

    // 2. Fallback: check src-tauri/binaries/ (dev mode, relative to cwd)
    for candidate in [
        std::path::PathBuf::from("src-tauri/binaries/mpv.app/Contents/MacOS/mpv"),
        std::path::PathBuf::from("src-tauri/binaries/mpv.exe"),
        std::path::PathBuf::from("binaries/mpv.app/Contents/MacOS/mpv"),
        std::path::PathBuf::from("binaries/mpv.exe"),
    ] {
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    // 3. Fallback: system PATH
    let system_name = if cfg!(target_os = "windows") {
        "mpv.exe"
    } else {
        "mpv"
    };
    if let Ok(output) = std::process::Command::new("which")
        .arg(system_name)
        .output()
        && output.status.success()
    {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() {
            return Ok(std::path::PathBuf::from(path));
        }
    }

    Err("mpv not found. Run 'make ensure-mpv' to download it.".to_string())
}

#[tauri::command]
pub fn play_file(path: String) -> Result<(), String> {
    let mpv = find_mpv()?;
    std::process::Command::new(&mpv)
        .args(["--force-window=yes", &path])
        .spawn()
        .map_err(|e| format!("Failed to start mpv: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn play_files(paths: Vec<String>) -> Result<(), String> {
    if paths.is_empty() {
        return Err("No files to play".to_string());
    }
    let mpv = find_mpv()?;
    let mut cmd = std::process::Command::new(&mpv);
    cmd.arg("--force-window=yes");
    for p in &paths {
        cmd.arg(p);
    }
    cmd.spawn()
        .map_err(|e| format!("Failed to start mpv: {e}"))?;
    Ok(())
}
