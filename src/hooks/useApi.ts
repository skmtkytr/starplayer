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
  description?: string,
  isSmart?: boolean
): Promise<Playlist> {
  return invoke("create_playlist", {
    name,
    description: description ?? null,
    isSmart: isSmart ?? false,
  });
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

export async function addToPlaylist(
  playlistId: string,
  mediaIds: string[]
): Promise<void> {
  return invoke("add_to_playlist", { playlistId, mediaIds });
}

export async function getPlaylistItems(
  playlistId: string
): Promise<MediaFile[]> {
  return invoke("get_playlist_items", { playlistId });
}

// === Filter API ===

export async function addPlaylistFilter(
  playlistId: string,
  filterType: string,
  operator: string,
  value: string
): Promise<void> {
  return invoke("add_playlist_filter", {
    playlistId,
    filterType,
    operator,
    value,
  });
}

export async function getPlaylistFilters(
  playlistId: string
): Promise<PlaylistFilter[]> {
  return invoke("get_playlist_filters", { playlistId });
}

// === Player API ===

export async function playFile(path: string): Promise<void> {
  return invoke("play_file", { path });
}

export async function playPlaylist(playlistId: string): Promise<void> {
  return invoke("play_playlist", { playlistId });
}
