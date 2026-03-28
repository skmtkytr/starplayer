export interface Workspace {
  id: string;
  name: string;
  path: string;
}

export interface MediaFile {
  id: string;
  workspace_id: string;
  path: string;
  filename: string;
  extension: string;
  size_bytes: number;
  duration_secs: number | null;
  width: number | null;
  height: number | null;
  series_name: string | null;
  series_number: number | null;
}

export interface PlaylistFilter {
  field: "filename" | "extension" | "path" | "series_name" | "workspace_id";
  operator: "contains" | "not_contains" | "equals" | "starts_with" | "ends_with";
  value: string;
}

export interface Playlist {
  id: string;
  name: string;
  playback_mode: "sequential" | "random" | "repeat";
  filters: PlaylistFilter[];
}

export interface ScanResult {
  added: number;
  updated: number;
  total: number;
}
