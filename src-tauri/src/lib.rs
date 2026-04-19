mod commands;
mod db;
mod models;
mod scanner;
mod watcher;

use db::Database;
use tauri::{Emitter, Manager};
use watcher::{MEDIA_CHANGED_EVENT, WatcherRegistry};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir");
            let db = Database::new(app_dir).expect("Failed to initialize database");
            app.manage(db);
            app.manage(WatcherRegistry::new());

            // Initial sync runs in the background so the UI can open
            // immediately even when workspaces live on slow network mounts.
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                let db = app_handle.state::<Database>();
                let registry = app_handle.state::<WatcherRegistry>();
                let workspaces = match db.list_workspaces() {
                    Ok(w) => w,
                    Err(e) => {
                        eprintln!("Failed to list workspaces for initial sync: {e}");
                        return;
                    }
                };
                for ws in workspaces {
                    if let Err(e) = scanner::scan_workspace(&db, &ws.id, &ws.path) {
                        eprintln!("Initial scan failed for {}: {e}", ws.name);
                    } else {
                        let _ = app_handle.emit(MEDIA_CHANGED_EVENT, &ws.id);
                    }
                    if let Err(e) =
                        registry.watch(ws.id.clone(), ws.path.clone(), app_handle.clone())
                    {
                        eprintln!("Failed to start watcher for {}: {e}", ws.name);
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::add_workspace,
            commands::list_workspaces,
            commands::remove_workspace,
            commands::scan_workspace,
            commands::list_media_files,
            commands::get_media_file,
            commands::create_playlist,
            commands::update_playlist,
            commands::list_playlists,
            commands::delete_playlist,
            commands::set_playback_mode,
            commands::get_playlist_files,
            commands::play_file,
            commands::play_files,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
