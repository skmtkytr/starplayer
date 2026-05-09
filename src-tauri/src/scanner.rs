use crate::db::Database;
use crate::models::{MediaFileInsert, ScanResult};
use std::collections::HashSet;
use std::path::Path;
use walkdir::WalkDir;

pub const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "m4v", "mpg", "mpeg", "ts", "vob", "ogv",
    "3gp", "f4v", "rm", "rmvb", "divx",
];

pub fn is_video_file(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => VIDEO_EXTENSIONS.contains(&ext.to_lowercase().as_str()),
        None => false,
    }
}

pub fn build_insert(workspace_id: &str, path: &Path) -> Option<MediaFileInsert> {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())?;
    if !VIDEO_EXTENSIONS.contains(&extension.as_str()) {
        return None;
    }
    let filename = path.file_name().and_then(|n| n.to_str())?.to_string();
    let full_path = path.to_string_lossy().to_string();
    let metadata = path.metadata().ok();
    let size = metadata.as_ref().map(|m| m.len() as i64).unwrap_or(0);
    let mtime = metadata
        .as_ref()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64);
    let (series_name, series_number) = extract_series_info(&filename);
    Some(MediaFileInsert {
        id: uuid::Uuid::new_v4().to_string(),
        workspace_id: workspace_id.to_string(),
        path: full_path,
        filename,
        extension,
        size_bytes: size,
        series_name,
        series_number,
        mtime,
    })
}

pub fn scan_workspace(
    db: &Database,
    workspace_id: &str,
    workspace_path: &str,
) -> Result<ScanResult, String> {
    let path = Path::new(workspace_path);
    if !path.exists() {
        return Err(format!("Path does not exist: {workspace_path}"));
    }

    let mut total = 0usize;
    let mut seen: HashSet<String> = HashSet::new();

    for entry in WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let file_path = entry.path();
        let Some(file) = build_insert(workspace_id, file_path) else {
            continue;
        };
        seen.insert(file.path.clone());
        if let Err(e) = db.upsert_media_file(&file) {
            eprintln!("Failed to insert media file: {e}");
        }
        total += 1;
    }

    // Remove DB entries whose paths are no longer present on disk.
    let known = db
        .list_workspace_media_paths(workspace_id)
        .map_err(|e| e.to_string())?;
    let mut removed = 0usize;
    for p in known {
        if !seen.contains(&p) {
            match db.remove_media_file_by_path(&p) {
                Ok(n) => removed += n,
                Err(e) => eprintln!("Failed to remove stale media row: {e}"),
            }
        }
    }

    Ok(ScanResult {
        added: total,
        updated: 0,
        removed,
        total,
    })
}

/// Extract series name and episode number from filename.
/// Supports patterns like:
///   "Series Name - 01.mp4"
///   "Series Name S01E02.mp4"
///   "Series Name EP03.mp4"
///   "Series Name #04.mp4"
fn extract_series_info(filename: &str) -> (Option<String>, Option<i32>) {
    let name_without_ext = filename
        .rsplit_once('.')
        .map(|(n, _)| n)
        .unwrap_or(filename);

    // Pattern: "Name - 01" or "Name - 001"
    if let Some((name, num_str)) = name_without_ext.rsplit_once(" - ")
        && let Ok(num) = num_str.trim().parse::<i32>()
    {
        return (Some(name.trim().to_string()), Some(num));
    }

    // Pattern: S01E02 or EP03 or #04
    let patterns = [
        (regex_lite_find(name_without_ext, r"[Ss]\d+[Ee](\d+)"), true),
        (regex_lite_find(name_without_ext, r"[Ee][Pp](\d+)"), false),
        (regex_lite_find(name_without_ext, r"#(\d+)"), false),
    ];

    for (result, _) in &patterns {
        if let Some((pos, num)) = result {
            let series = name_without_ext[..*pos]
                .trim()
                .trim_end_matches(['-', '_', ' '])
                .to_string();
            if !series.is_empty() {
                return (Some(series), Some(*num));
            }
        }
    }

    (None, None)
}

/// Simple regex-like pattern matching without pulling in the regex crate.
fn regex_lite_find(s: &str, _pattern: &str) -> Option<(usize, i32)> {
    // Pattern: S01E02
    if _pattern.contains("[Ss]") && _pattern.contains("[Ee]") {
        for (i, _) in s.char_indices() {
            let rest = &s[i..];
            if rest.len() >= 4 {
                let first = rest.as_bytes()[0];
                if (first == b'S' || first == b's')
                    && let Some(e_pos) = rest[1..].find(['E', 'e'])
                {
                    let season_str = &rest[1..1 + e_pos];
                    let after_e = &rest[2 + e_pos..];
                    if season_str.chars().all(|c| c.is_ascii_digit()) && !season_str.is_empty() {
                        let ep_len = after_e.chars().take_while(|c| c.is_ascii_digit()).count();
                        if ep_len > 0
                            && let Ok(num) = after_e[..ep_len].parse::<i32>()
                        {
                            return Some((i, num));
                        }
                    }
                }
            }
        }
    }

    // Pattern: EP03
    if _pattern.contains("[Ee][Pp]") {
        for (i, _) in s.char_indices() {
            let rest = &s[i..];
            if rest.len() >= 3 {
                let b0 = rest.as_bytes()[0];
                let b1 = rest.as_bytes()[1];
                if (b0 == b'E' || b0 == b'e') && (b1 == b'P' || b1 == b'p') {
                    let after = &rest[2..];
                    let num_len = after.chars().take_while(|c| c.is_ascii_digit()).count();
                    if num_len > 0
                        && let Ok(num) = after[..num_len].parse::<i32>()
                    {
                        return Some((i, num));
                    }
                }
            }
        }
    }

    // Pattern: #04
    if _pattern.contains('#') {
        for (i, c) in s.char_indices() {
            if c == '#' {
                let after = &s[i + 1..];
                let num_len = after.chars().take_while(|c| c.is_ascii_digit()).count();
                if num_len > 0
                    && let Ok(num) = after[..num_len].parse::<i32>()
                {
                    return Some((i, num));
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_series_dash_number() {
        let (name, num) = extract_series_info("My Series - 01.mp4");
        assert_eq!(name.as_deref(), Some("My Series"));
        assert_eq!(num, Some(1));
    }

    #[test]
    fn test_extract_series_s_e_pattern() {
        let (name, num) = extract_series_info("ShowName S01E05.mkv");
        assert_eq!(name.as_deref(), Some("ShowName"));
        assert_eq!(num, Some(5));
    }

    #[test]
    fn test_extract_series_ep_pattern() {
        let (name, num) = extract_series_info("Anime EP12.mp4");
        assert_eq!(name.as_deref(), Some("Anime"));
        assert_eq!(num, Some(12));
    }

    #[test]
    fn test_extract_series_hash_pattern() {
        let (name, num) = extract_series_info("Series #03.mp4");
        assert_eq!(name.as_deref(), Some("Series"));
        assert_eq!(num, Some(3));
    }

    #[test]
    fn test_extract_no_series() {
        let (name, num) = extract_series_info("random_video.mp4");
        assert_eq!(name, None);
        assert_eq!(num, None);
    }

    #[test]
    fn is_video_file_matches_known_extensions() {
        assert!(is_video_file(std::path::Path::new("/tmp/a.mp4")));
        assert!(is_video_file(std::path::Path::new("/tmp/a.MKV")));
        assert!(!is_video_file(std::path::Path::new("/tmp/a.txt")));
        assert!(!is_video_file(std::path::Path::new("/tmp/no_ext")));
    }

    #[test]
    fn build_insert_returns_none_for_non_video() {
        let p = std::path::Path::new("/tmp/a.txt");
        assert!(build_insert("ws", p).is_none());
    }

    #[test]
    fn build_insert_populates_mtime_from_filesystem() {
        let tmp = std::env::temp_dir().join(format!("sp-mtime-{}.mp4", uuid::Uuid::new_v4()));
        std::fs::write(&tmp, b"x").unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let insert = build_insert("ws", &tmp).expect("build_insert");
        let mtime = insert.mtime.expect("mtime should be present for existing file");
        // Allow ±60s skew between syscall and our `now`.
        assert!(
            (mtime - now).abs() < 60,
            "mtime {mtime} should be near now {now}"
        );

        std::fs::remove_file(&tmp).ok();
    }

    #[test]
    fn scan_workspace_prunes_missing_files() {
        use crate::db::Database;

        let tmp = std::env::temp_dir().join(format!("starplayer-scan-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&tmp).unwrap();
        let db = Database::new(tmp.join("_db")).unwrap();
        let ws = db.add_workspace("t", tmp.to_str().unwrap()).unwrap();

        let kept = tmp.join("kept.mp4");
        let gone = tmp.join("gone.mp4");
        std::fs::write(&kept, b"x").unwrap();
        std::fs::write(&gone, b"x").unwrap();

        let r1 = scan_workspace(&db, &ws.id, tmp.to_str().unwrap()).unwrap();
        assert_eq!(r1.total, 2);
        assert_eq!(r1.removed, 0);

        std::fs::remove_file(&gone).unwrap();

        let r2 = scan_workspace(&db, &ws.id, tmp.to_str().unwrap()).unwrap();
        assert_eq!(r2.total, 1);
        assert_eq!(r2.removed, 1);

        let paths = db.list_workspace_media_paths(&ws.id).unwrap();
        assert_eq!(paths.len(), 1);
        assert!(paths[0].ends_with("kept.mp4"));

        std::fs::remove_dir_all(&tmp).ok();
    }

}
