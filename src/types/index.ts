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

export interface Playlist {
  id: string;
  name: string;
  description: string | null;
  is_smart: boolean;
  playback_mode: "sequential" | "random" | "repeat";
  item_count: number;
}

export interface PlaylistFilter {
  filter_type: string;
  operator: string;
  value: string;
}

export interface ScanResult {
  added: number;
  updated: number;
  total: number;
}
