use std::io::{Read, Seek, SeekFrom};
use std::sync::atomic::{AtomicU16, Ordering};
use std::thread;

static PORT: AtomicU16 = AtomicU16::new(0);

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

pub fn start() -> u16 {
    let port = portpicker::pick_unused_port().expect("No free port available");
    PORT.store(port, Ordering::SeqCst);

    thread::spawn(move || {
        let server =
            tiny_http::Server::http(format!("127.0.0.1:{port}")).expect("Failed to start server");

        for request in server.incoming_requests() {
            let url = request.url().to_string();
            // URL format: /stream/<url_encoded_path>
            let encoded_path = url.strip_prefix("/stream/").unwrap_or("");
            let path = urlencoding::decode(encoded_path)
                .unwrap_or_default()
                .to_string();

            if path.is_empty() {
                let resp = tiny_http::Response::from_string("Missing path")
                    .with_status_code(tiny_http::StatusCode(400));
                request.respond(resp).ok();
                continue;
            }

            let file = match std::fs::File::open(&path) {
                Ok(f) => f,
                Err(e) => {
                    let resp = tiny_http::Response::from_string(format!("Not found: {e}"))
                        .with_status_code(tiny_http::StatusCode(404));
                    request.respond(resp).ok();
                    continue;
                }
            };

            let metadata = match file.metadata() {
                Ok(m) => m,
                Err(_) => {
                    let resp = tiny_http::Response::from_string("Cannot read metadata")
                        .with_status_code(tiny_http::StatusCode(500));
                    request.respond(resp).ok();
                    continue;
                }
            };

            let total_size = metadata.len();
            let ext = std::path::Path::new(&path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            let mime = mime_from_ext(&ext);
            let content_type = tiny_http::Header::from_bytes("Content-Type", mime).unwrap();
            let cors = tiny_http::Header::from_bytes("Access-Control-Allow-Origin", "*").unwrap();
            let accept_ranges = tiny_http::Header::from_bytes("Accept-Ranges", "bytes").unwrap();

            // Parse Range header
            let range_header = request
                .headers()
                .iter()
                .find(|h| h.field.as_str() == "Range" || h.field.as_str() == "range")
                .map(|h| h.value.as_str().to_string());

            if let Some(ref range_str) = range_header
                && let Some((start, end)) = parse_range(range_str, total_size)
            {
                    let length = end - start + 1;
                    let mut file = file;
                    if file.seek(SeekFrom::Start(start)).is_err() {
                        let resp = tiny_http::Response::from_string("Seek error")
                            .with_status_code(tiny_http::StatusCode(500));
                        request.respond(resp).ok();
                        continue;
                    }
                    let mut buf = vec![0u8; length as usize];
                    if file.read_exact(&mut buf).is_err() {
                        let resp = tiny_http::Response::from_string("Read error")
                            .with_status_code(tiny_http::StatusCode(500));
                        request.respond(resp).ok();
                        continue;
                    }

                    let content_range = tiny_http::Header::from_bytes(
                        "Content-Range",
                        format!("bytes {start}-{end}/{total_size}"),
                    )
                    .unwrap();

                    let resp = tiny_http::Response::new(
                        tiny_http::StatusCode(206),
                        vec![content_type, cors, accept_ranges, content_range],
                        std::io::Cursor::new(buf),
                        Some(length as usize),
                        None,
                    );
                    request.respond(resp).ok();
                    continue;
            }

            // No range or invalid range: return full file
            let mut file = file;
            let mut buf = Vec::new();
            if file.read_to_end(&mut buf).is_err() {
                let resp = tiny_http::Response::from_string("Read error")
                    .with_status_code(tiny_http::StatusCode(500));
                request.respond(resp).ok();
                continue;
            }

            let resp = tiny_http::Response::new(
                tiny_http::StatusCode(200),
                vec![content_type, cors, accept_ranges],
                std::io::Cursor::new(buf),
                Some(total_size as usize),
                None,
            );
            request.respond(resp).ok();
        }
    });

    port
}

pub fn get_port() -> u16 {
    PORT.load(Ordering::SeqCst)
}

fn parse_range(header: &str, total: u64) -> Option<(u64, u64)> {
    let header = header.strip_prefix("bytes=")?;
    let parts: Vec<&str> = header.splitn(2, '-').collect();
    if parts.len() != 2 {
        return None;
    }
    let start: u64 = parts[0].parse().ok()?;
    let end: u64 = if parts[1].is_empty() {
        std::cmp::min(start + 2 * 1024 * 1024 - 1, total - 1)
    } else {
        parts[1].parse().ok()?
    };
    if start > end || start >= total {
        return None;
    }
    Some((start, std::cmp::min(end, total - 1)))
}
