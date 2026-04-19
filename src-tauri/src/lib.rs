mod commands;
mod db;
mod models;
mod scanner;
mod watcher;

use db::Database;
use tauri::Manager;
use watcher::WatcherRegistry;

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
            let registry = WatcherRegistry::new();

            // Initial sync + start watcher for each existing workspace.
            if let Ok(workspaces) = db.list_workspaces() {
                for ws in &workspaces {
                    if let Err(e) = scanner::scan_workspace(&db, &ws.id, &ws.path) {
                        eprintln!("Initial scan failed for {}: {e}", ws.name);
                    }
                    if let Err(e) =
                        registry.watch(ws.id.clone(), ws.path.clone(), app.handle().clone())
                    {
                        eprintln!("Failed to start watcher for {}: {e}", ws.name);
                    }
                }
            }

            app.manage(db);
            app.manage(registry);
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
