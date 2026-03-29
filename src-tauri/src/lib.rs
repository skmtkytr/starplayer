mod commands;
mod db;
mod models;
mod scanner;

use db::Database;
use std::io::{Read, Seek, SeekFrom};
use tauri::Manager;

fn mime_from_ext(ext: &str) -> &str {
    match ext {
        "mp4" | "m4v" => "video/mp4",
        "mkv" => "video/x-matroska",
        "webm" => "video/webm",
        "avi" => "video/x-msvideo",
        "mov" => "video/quicktime",
        "wmv" => "video/x-ms-wmv",
        "flv" => "video/x-flv",
        "mpg" | "mpeg" => "video/mpeg",
        "ts" => "video/mp2t",
        "ogv" => "video/ogg",
        "3gp" => "video/3gpp",
        _ => "application/octet-stream",
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .register_asynchronous_uri_scheme_protocol("stream", |_ctx, request, responder| {
            std::thread::spawn(move || {
                let uri = request.uri().to_string();
                // URI format varies by platform:
                //   macOS/Linux: stream://localhost/<encoded_path>
                //   Windows:     https://stream.localhost/<encoded_path>
                let path = uri
                    .strip_prefix("stream://localhost/")
                    .or_else(|| uri.strip_prefix("stream://localhost"))
                    .or_else(|| uri.strip_prefix("https://stream.localhost/"))
                    .or_else(|| uri.strip_prefix("https://stream.localhost"))
                    .or_else(|| uri.strip_prefix("http://stream.localhost/"))
                    .or_else(|| uri.strip_prefix("http://stream.localhost"))
                    .unwrap_or("");
                let path = urlencoding::decode(path).unwrap_or_default().to_string();

                if path.is_empty() {
                    responder.respond(
                        tauri::http::Response::builder()
                            .status(400)
                            .body(b"Missing path".to_vec())
                            .unwrap(),
                    );
                    return;
                }

                let file = match std::fs::File::open(&path) {
                    Ok(f) => f,
                    Err(e) => {
                        responder.respond(
                            tauri::http::Response::builder()
                                .status(404)
                                .body(format!("File not found: {e}").into_bytes())
                                .unwrap(),
                        );
                        return;
                    }
                };

                let metadata = match file.metadata() {
                    Ok(m) => m,
                    Err(_) => {
                        responder.respond(
                            tauri::http::Response::builder()
                                .status(500)
                                .body(b"Cannot read metadata".to_vec())
                                .unwrap(),
                        );
                        return;
                    }
                };

                let total_size = metadata.len();
                let ext = std::path::Path::new(&path)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                let mime = mime_from_ext(&ext);

                // Parse Range header
                let range_header = request
                    .headers()
                    .get("range")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("");

                fn build_response(
                    status: u16, mime: &str, body: Vec<u8>, content_len: u64,
                    extra_headers: &[(&str, &str)],
                ) -> tauri::http::Response<Vec<u8>> {
                    let mut builder = tauri::http::Response::builder()
                        .status(status)
                        .header("Content-Type", mime)
                        .header("Content-Length", content_len.to_string())
                        .header("Accept-Ranges", "bytes")
                        .header("Access-Control-Allow-Origin", "*");
                    for (k, v) in extra_headers {
                        builder = builder.header(*k, *v);
                    }
                    builder.body(body).unwrap()
                }

                if let Some(range) = parse_range(range_header, total_size) {
                    let (start, end) = range;
                    let length = end - start + 1;
                    let mut file = file;
                    if let Err(e) = file.seek(SeekFrom::Start(start)) {
                        responder.respond(build_response(500, mime, format!("Seek error: {e}").into_bytes(), 0, &[]));
                        return;
                    }
                    let mut buf = vec![0u8; length as usize];
                    if let Err(e) = file.read_exact(&mut buf) {
                        responder.respond(build_response(500, mime, format!("Read error: {e}").into_bytes(), 0, &[]));
                        return;
                    }
                    let range_val = format!("bytes {start}-{end}/{total_size}");
                    responder.respond(build_response(
                        206, mime, buf, length,
                        &[("Content-Range", &range_val)],
                    ));
                } else {
                    let mut file = file;
                    let mut buf = Vec::new();
                    if let Err(e) = file.read_to_end(&mut buf) {
                        responder.respond(build_response(500, mime, format!("Read error: {e}").into_bytes(), 0, &[]));
                        return;
                    }
                    responder.respond(build_response(200, mime, buf, total_size, &[]));
                }
            });
        })
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn parse_range(header: &str, total: u64) -> Option<(u64, u64)> {
    let header = header.strip_prefix("bytes=")?;
    let parts: Vec<&str> = header.splitn(2, '-').collect();
    if parts.len() != 2 {
        return None;
    }
    let start: u64 = parts[0].parse().ok()?;
    let end: u64 = if parts[1].is_empty() {
        // "bytes=0-" means from start to end, but cap chunk size
        std::cmp::min(start + 2 * 1024 * 1024 - 1, total - 1)
    } else {
        parts[1].parse().ok()?
    };
    if start > end || start >= total {
        return None;
    }
    Some((start, std::cmp::min(end, total - 1)))
}
