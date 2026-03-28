import { invoke } from "@tauri-apps/api/core";
import type {
  Workspace,
  MediaFile,
  Playlist,
  PlaylistFilter,
  ScanResult,
} from "../types";

// === Workspace API ===

export async function addWorkspace(
  name: string,
  path: string
): Promise<Workspace> {
  return invoke("add_workspace", { name, path });
}

export async function listWorkspaces(): Promise<Workspace[]> {
  return invoke("list_workspaces");
}

export async function removeWorkspace(id: string): Promise<void> {
  return invoke("remove_workspace", { id });
}

export async function scanWorkspace(
  workspaceId: string,
  path: string
): Promise<ScanResult> {
  return invoke("scan_workspace", { workspaceId, path });
}

// === Media API ===

export async function listMediaFiles(params?: {
  workspaceId?: string;
  search?: string;
  extension?: string;
}): Promise<MediaFile[]> {
  return invoke("list_media_files", {
    workspaceId: params?.workspaceId ?? null,
    search: params?.search ?? null,
    extension: params?.extension ?? null,
  });
}

export async function getMediaFile(id: string): Promise<MediaFile | null> {
  return invoke("get_media_file", { id });
}

// === Playlist API ===

export async function createPlaylist(
  name: string,
  filters: PlaylistFilter[]
): Promise<Playlist> {
  return invoke("create_playlist", { name, filters });
}

export async function updatePlaylist(
  id: string,
  name: string,
  filters: PlaylistFilter[]
): Promise<void> {
  return invoke("update_playlist", { id, name, filters });
}

export async function listPlaylists(): Promise<Playlist[]> {
  return invoke("list_playlists");
}

export async function deletePlaylist(id: string): Promise<void> {
  return invoke("delete_playlist", { id });
}

export async function setPlaybackMode(
  playlistId: string,
  mode: string
): Promise<void> {
  return invoke("set_playback_mode", { playlistId, mode });
}

export async function getPlaylistFiles(
  playlistId: string
): Promise<MediaFile[]> {
  return invoke("get_playlist_files", { playlistId });
}
