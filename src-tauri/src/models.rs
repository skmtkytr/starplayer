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

/// A playlist is a named set of filter conditions.
/// When viewed, it dynamically queries media_files matching those filters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub playback_mode: String,
    pub filters: Vec<PlaylistFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistFilter {
    /// "filename", "extension", "path", "series_name", "workspace_id"
    pub field: String,
    /// "contains", "not_contains", "equals", "starts_with", "ends_with"
    pub operator: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub added: usize,
    pub updated: usize,
    pub removed: usize,
    pub total: usize,
}
