mod commands;
mod db;
mod models;
mod scanner;

use db::Database;
use tauri::Manager;

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
