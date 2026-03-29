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

fn find_vlc() -> Result<std::path::PathBuf, String> {
    // 1. Check bundled VLC next to the executable
    if let Ok(exe) = std::env::current_exe() {
        let exe_dir = exe.parent().unwrap_or(std::path::Path::new("."));

        for candidate in [
            // macOS dev: src-tauri/target/debug/ → ../../binaries/vlc/Contents/MacOS/VLC
            exe_dir.join("../../binaries/vlc/Contents/MacOS/VLC"),
            // macOS bundled: .app/Contents/MacOS/ → ../Resources/vlc/Contents/MacOS/VLC
            exe_dir.join("../Resources/vlc/Contents/MacOS/VLC"),
            exe_dir.join("binaries/vlc/Contents/MacOS/VLC"),
            // Windows dev: src-tauri/target/debug/ → ../../binaries/vlc/vlc.exe
            exe_dir.join("../../binaries/vlc/vlc.exe"),
            // Windows bundled
            exe_dir.join("vlc/vlc.exe"),
            exe_dir.join("binaries/vlc/vlc.exe"),
        ] {
            if candidate.exists() {
                return Ok(candidate);
            }
        }
    }

    // 2. Fallback: relative to cwd (dev mode)
    for candidate in [
        std::path::PathBuf::from("src-tauri/binaries/vlc/Contents/MacOS/VLC"),
        std::path::PathBuf::from("src-tauri/binaries/vlc/vlc.exe"),
        std::path::PathBuf::from("binaries/vlc/Contents/MacOS/VLC"),
        std::path::PathBuf::from("binaries/vlc/vlc.exe"),
    ] {
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    // 3. Fallback: system-installed VLC
    if cfg!(target_os = "macos") {
        let system = std::path::PathBuf::from("/Applications/VLC.app/Contents/MacOS/VLC");
        if system.exists() {
            return Ok(system);
        }
    } else if cfg!(target_os = "windows") {
        for candidate in [
            r"C:\Program Files\VideoLAN\VLC\vlc.exe",
            r"C:\Program Files (x86)\VideoLAN\VLC\vlc.exe",
        ] {
            let p = std::path::PathBuf::from(candidate);
            if p.exists() {
                return Ok(p);
            }
        }
    } else {
        // Linux: check PATH
        if let Ok(output) = std::process::Command::new("which")
            .arg("vlc")
            .output()
            && output.status.success()
        {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Ok(std::path::PathBuf::from(path));
            }
        }
    }

    Err("VLC not found. Run 'make ensure-vlc' to download it.".to_string())
}

#[tauri::command]
pub fn play_file(path: String) -> Result<(), String> {
    let vlc = find_vlc()?;
    std::process::Command::new(&vlc)
        .args(["--started-from-file", &path])
        .spawn()
        .map_err(|e| format!("Failed to start VLC: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn play_files(paths: Vec<String>) -> Result<(), String> {
    if paths.is_empty() {
        return Err("No files to play".to_string());
    }
    let vlc = find_vlc()?;
    let mut cmd = std::process::Command::new(&vlc);
    cmd.arg("--started-from-file");
    for p in &paths {
        cmd.arg(p);
    }
    cmd.spawn()
        .map_err(|e| format!("Failed to start VLC: {e}"))?;
    Ok(())
}
