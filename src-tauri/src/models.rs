use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaFile {
    pub id: String,
    pub workspace_id: String,
    pub path: String,
    pub filename: String,
    pub extension: String,
    pub size_bytes: i64,
    pub duration_secs: Option<f64>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub series_name: Option<String>,
    pub series_number: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaFileInsert {
    pub id: String,
    pub workspace_id: String,
    pub path: String,
    pub filename: String,
    pub extension: String,
    pub size_bytes: i64,
    pub series_name: Option<String>,
    pub series_number: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_smart: bool,
    pub playback_mode: String,
    pub item_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistFilter {
    pub filter_type: String,
    pub operator: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub added: usize,
    pub updated: usize,
    pub total: usize,
}
