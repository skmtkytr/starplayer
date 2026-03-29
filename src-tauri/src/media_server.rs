use std::io::{Read, Seek, SeekFrom};
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;
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

    let server = Arc::new(
        tiny_http::Server::http(format!("127.0.0.1:{port}")).expect("Failed to start server"),
    );

    for _ in 0..4 {
        let server = server.clone();
        thread::spawn(move || loop {
            let request = match server.recv() {
                Ok(r) => r,
                Err(_) => break,
            };
            handle_request(request);
        });
    }

    port
}

pub fn get_port() -> u16 {
    PORT.load(Ordering::SeqCst)
}

fn handle_request(request: tiny_http::Request) {
    let url = request.url().to_string();
    let encoded_path = url.strip_prefix("/stream/").unwrap_or("");
    let path = urlencoding::decode(encoded_path)
        .unwrap_or_default()
        .to_string();

    if path.is_empty() {
        let resp = tiny_http::Response::from_string("Missing path")
            .with_status_code(tiny_http::StatusCode(400));
        request.respond(resp).ok();
        return;
    }

    let file = match std::fs::File::open(&path) {
        Ok(f) => f,
        Err(e) => {
            let resp = tiny_http::Response::from_string(format!("Not found: {e}"))
                .with_status_code(tiny_http::StatusCode(404));
            request.respond(resp).ok();
            return;
        }
    };

    let total_size = match file.metadata() {
        Ok(m) => m.len(),
        Err(_) => {
            let resp = tiny_http::Response::from_string("Cannot read metadata")
                .with_status_code(tiny_http::StatusCode(500));
            request.respond(resp).ok();
            return;
        }
    };

    let ext = std::path::Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let mime = mime_from_ext(&ext);
    let content_type = tiny_http::Header::from_bytes("Content-Type", mime).unwrap();
    let cors = tiny_http::Header::from_bytes("Access-Control-Allow-Origin", "*").unwrap();
    let accept_ranges = tiny_http::Header::from_bytes("Accept-Ranges", "bytes").unwrap();

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
            return;
        }

        let content_range = tiny_http::Header::from_bytes(
            "Content-Range",
            format!("bytes {start}-{end}/{total_size}"),
        )
        .unwrap();

        let reader = file.take(length);
        let resp = tiny_http::Response::new(
            tiny_http::StatusCode(206),
            vec![content_type, cors, accept_ranges, content_range],
            reader,
            Some(length as usize),
            None,
        );
        request.respond(resp).ok();
        return;
    }

    // No range: stream full file directly
    let resp = tiny_http::Response::new(
        tiny_http::StatusCode(200),
        vec![content_type, cors, accept_ranges],
        file,
        Some(total_size as usize),
        None,
    );
    request.respond(resp).ok();
}

fn parse_range(header: &str, total: u64) -> Option<(u64, u64)> {
    let header = header.strip_prefix("bytes=")?;
    let parts: Vec<&str> = header.splitn(2, '-').collect();
    if parts.len() != 2 {
        return None;
    }
    let start: u64 = parts[0].parse().ok()?;
    let end: u64 = if parts[1].is_empty() {
        total - 1
    } else {
        parts[1].parse().ok()?
    };
    if start > end || start >= total {
        return None;
    }
    Some((start, std::cmp::min(end, total - 1)))
}
