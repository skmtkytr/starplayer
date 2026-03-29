import { useState, useEffect, useCallback, useRef } from "react";
import type { Workspace, MediaFile, Playlist, PlaylistFilter } from "./types";
import {
  listWorkspaces,
  addWorkspace,
  removeWorkspace,
  scanWorkspace,
  listMediaFiles,
  listPlaylists,
  createPlaylist,
  updatePlaylist,
  deletePlaylist,
  getPlaylistFiles,
  setPlaybackMode,
} from "./hooks/useApi";
import { WorkspaceDialog } from "./components/WorkspaceDialog";
import { PlaylistDialog } from "./components/PlaylistDialog";
import { VideoPlayer } from "./components/VideoPlayer";
import React from "react";

const MemoizedVideoPlayer = React.memo(VideoPlayer);

type View =
  | { type: "library"; workspaceId?: string }
  | { type: "playlist"; playlistId: string };

function formatFilter(
  f: PlaylistFilter,
  workspaces: Workspace[],
  verbose: boolean
): string {
  if (f.field === "workspace_id") {
    const ws = workspaces.find((w) => w.id === f.value);
    return verbose ? `workspace: ${ws?.name ?? "?"}` : ws?.name ?? "workspace";
  }
  return verbose ? `${f.field} ${f.operator} "${f.value}"` : f.value;
}

function App() {
  const [workspaces, setWorkspaces] = useState<Workspace[]>([]);
  const [playlists, setPlaylists] = useState<Playlist[]>([]);
  const [mediaFiles, setMediaFiles] = useState<MediaFile[]>([]);
  const [currentView, setCurrentView] = useState<View>({ type: "library" });
  const [searchQuery, setSearchQuery] = useState("");
  const [extensionFilter, setExtensionFilter] = useState("");
  const [selectedFiles, setSelectedFiles] = useState<Set<string>>(new Set());
  const [showWorkspaceDialog, setShowWorkspaceDialog] = useState(false);
  const [showPlaylistDialog, setShowPlaylistDialog] = useState(false);
  const [editingPlaylist, setEditingPlaylist] = useState<Playlist | undefined>();
  const [statusMessage, setStatusMessage] = useState("");
  const [playingFile, setPlayingFile] = useState<MediaFile | null>(null);
  const [playQueue, setPlayQueue] = useState<MediaFile[]>([]);
  const [playQueueIndex, setPlayQueueIndex] = useState(0);
  const mediaListRef = useRef<HTMLDivElement>(null);

  const loadWorkspaces = useCallback(async () => {
    try {
      setWorkspaces(await listWorkspaces());
    } catch (e) {
      console.error("Failed to load workspaces:", e);
    }
  }, []);

  const loadPlaylists = useCallback(async () => {
    try {
      setPlaylists(await listPlaylists());
    } catch (e) {
      console.error("Failed to load playlists:", e);
    }
  }, []);

  const loadMediaFiles = useCallback(async () => {
    try {
      if (currentView.type === "playlist") {
        setMediaFiles(await getPlaylistFiles(currentView.playlistId));
      } else {
        setMediaFiles(
          await listMediaFiles({
            workspaceId: currentView.workspaceId,
            search: searchQuery || undefined,
            extension: extensionFilter || undefined,
          })
        );
      }
    } catch (e) {
      console.error("Failed to load media files:", e);
    }
  }, [currentView, searchQuery, extensionFilter]);

  useEffect(() => {
    loadWorkspaces();
    loadPlaylists();
  }, [loadWorkspaces, loadPlaylists]);

  useEffect(() => {
    loadMediaFiles();
  }, [loadMediaFiles]);

  // Ctrl+A / Cmd+A
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "a") {
        const tag = (e.target as HTMLElement).tagName;
        if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return;
        e.preventDefault();
        setSelectedFiles((prev) => {
          if (prev.size === mediaFiles.length && mediaFiles.length > 0) {
            return new Set();
          }
          return new Set(mediaFiles.map((f) => f.id));
        });
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [mediaFiles]);

  const handleAddWorkspace = async (name: string, path: string) => {
    try {
      const ws = await addWorkspace(name, path);
      setStatusMessage("Scanning...");
      const result = await scanWorkspace(ws.id, ws.path);
      setStatusMessage(`Scan complete: ${result.total} files found`);
      await loadWorkspaces();
      await loadMediaFiles();
      setShowWorkspaceDialog(false);
    } catch (e) {
      setStatusMessage(`Error: ${e}`);
    }
  };

  const handleRemoveWorkspace = async (id: string) => {
    try {
      await removeWorkspace(id);
      await loadWorkspaces();
      if (currentView.type === "library" && currentView.workspaceId === id) {
        setCurrentView({ type: "library" });
      }
      await loadMediaFiles();
    } catch (e) {
      setStatusMessage(`Error: ${e}`);
    }
  };

  const handleRescan = async (ws: Workspace) => {
    try {
      setStatusMessage(`Scanning ${ws.name}...`);
      const result = await scanWorkspace(ws.id, ws.path);
      setStatusMessage(`Scan complete: ${result.total} files found`);
      await loadMediaFiles();
    } catch (e) {
      setStatusMessage(`Error: ${e}`);
    }
  };

  const handleCreatePlaylist = async (
    name: string,
    filters: PlaylistFilter[]
  ) => {
    try {
      if (editingPlaylist) {
        await updatePlaylist(editingPlaylist.id, name, filters);
      } else {
        await createPlaylist(name, filters);
      }
      await loadPlaylists();
      // Refresh if we're viewing this playlist
      if (
        editingPlaylist &&
        currentView.type === "playlist" &&
        currentView.playlistId === editingPlaylist.id
      ) {
        await loadMediaFiles();
      }
      setShowPlaylistDialog(false);
      setEditingPlaylist(undefined);
    } catch (e) {
      setStatusMessage(`Error: ${e}`);
    }
  };

  const handleDeletePlaylist = async (id: string) => {
    try {
      await deletePlaylist(id);
      await loadPlaylists();
      if (currentView.type === "playlist" && currentView.playlistId === id) {
        setCurrentView({ type: "library" });
      }
    } catch (e) {
      setStatusMessage(`Error: ${e}`);
    }
  };

  const handlePlayFile = (file: MediaFile) => {
    setPlayingFile(file);
    setPlayQueue(mediaFiles);
    setPlayQueueIndex(mediaFiles.findIndex((f) => f.id === file.id));
  };

  const handlePlayNext = useCallback(() => {
    setPlayQueueIndex((prev) => {
      const next = (prev + 1) % playQueue.length;
      setPlayingFile(playQueue[next] ?? null);
      return next;
    });
  }, [playQueue]);

  const handlePlayPrev = useCallback(() => {
    setPlayQueueIndex((prev) => {
      const prevIdx = (prev - 1 + playQueue.length) % playQueue.length;
      setPlayingFile(playQueue[prevIdx] ?? null);
      return prevIdx;
    });
  }, [playQueue]);

  const handleClosePlayer = useCallback(() => {
    setPlayingFile(null);
  }, []);

  const handlePlayAll = (shuffle: boolean) => {
    if (mediaFiles.length === 0) return;
    let queue = [...mediaFiles];
    if (shuffle) {
      for (let i = queue.length - 1; i > 0; i--) {
        const j = Math.floor(Math.random() * (i + 1));
        [queue[i], queue[j]] = [queue[j], queue[i]];
      }
    }
    setPlayQueue(queue);
    setPlayQueueIndex(0);
    setPlayingFile(queue[0]);
  };

  const handleSetPlaybackMode = async (playlistId: string, mode: string) => {
    try {
      await setPlaybackMode(playlistId, mode);
      await loadPlaylists();
    } catch (e) {
      setStatusMessage(`Error: ${e}`);
    }
  };

  const toggleFileSelection = (id: string) => {
    setSelectedFiles((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  };

  const selectAll = () => {
    if (selectedFiles.size === mediaFiles.length) {
      setSelectedFiles(new Set());
    } else {
      setSelectedFiles(new Set(mediaFiles.map((f) => f.id)));
    }
  };

  const formatSize = (bytes: number) => {
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
    if (bytes < 1024 * 1024 * 1024)
      return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
  };

  const extensions = [...new Set(mediaFiles.map((f) => f.extension))].sort();
  const currentPlaylist =
    currentView.type === "playlist"
      ? playlists.find((p) => p.id === currentView.playlistId)
      : null;
  const allSelected =
    mediaFiles.length > 0 && selectedFiles.size === mediaFiles.length;

  return (
    <div className="app-layout">
      {/* Sidebar */}
      <div className="sidebar">
        <div className="sidebar-header">
          <h1>Star Player</h1>
        </div>

        <div className="sidebar-section">
          <h3>Library</h3>
          <div
            className={`sidebar-item ${currentView.type === "library" && !currentView.workspaceId ? "active" : ""}`}
            onClick={() => setCurrentView({ type: "library" })}
          >
            All Media
          </div>
        </div>

        <div className="sidebar-section">
          <h3>Workspaces</h3>
          {workspaces.map((ws) => (
            <div
              key={ws.id}
              className={`sidebar-item ${currentView.type === "library" && currentView.workspaceId === ws.id ? "active" : ""}`}
              onClick={() =>
                setCurrentView({ type: "library", workspaceId: ws.id })
              }
              onContextMenu={(e) => {
                e.preventDefault();
                if (confirm(`Remove workspace "${ws.name}"?`)) {
                  handleRemoveWorkspace(ws.id);
                }
              }}
              onDoubleClick={() => handleRescan(ws)}
            >
              {ws.name}
            </div>
          ))}
          <div
            className="sidebar-item accent"
            onClick={() => setShowWorkspaceDialog(true)}
          >
            + Add Workspace
          </div>
        </div>

        <div className="sidebar-section">
          <h3>Playlists</h3>
          {playlists.map((pl) => (
            <div
              key={pl.id}
              className={`sidebar-item ${currentView.type === "playlist" && currentView.playlistId === pl.id ? "active" : ""}`}
              onClick={() =>
                setCurrentView({ type: "playlist", playlistId: pl.id })
              }
              onContextMenu={(e) => {
                e.preventDefault();
                if (confirm(`Delete playlist "${pl.name}"?`)) {
                  handleDeletePlaylist(pl.id);
                }
              }}
              onDoubleClick={() => {
                setEditingPlaylist(pl);
                setShowPlaylistDialog(true);
              }}
            >
              {pl.name}
              <span className="count">
                {pl.filters
                  .map((f) => formatFilter(f, workspaces, false))
                  .join(", ") || "no filters"}
              </span>
            </div>
          ))}
          <div
            className="sidebar-item accent"
            onClick={() => {
              setEditingPlaylist(undefined);
              setShowPlaylistDialog(true);
            }}
          >
            + New Playlist
          </div>
        </div>
      </div>

      {/* Main content */}
      <div className="main-content">
        <div className="toolbar">
          {currentView.type === "library" && (
            <>
              <input
                className="search-input"
                placeholder="Search files..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                autoCapitalize="off"
                autoCorrect="off"
              />
              <select
                className="filter-select"
                value={extensionFilter}
                onChange={(e) => setExtensionFilter(e.target.value)}
              >
                <option value="">All formats</option>
                {extensions.map((ext) => (
                  <option key={ext} value={ext}>
                    .{ext}
                  </option>
                ))}
              </select>
              {mediaFiles.length > 0 && (
                <>
                  <button
                    className="btn btn-primary btn-small"
                    onClick={() => handlePlayAll(false)}
                  >
                    Play
                  </button>
                  <button
                    className="btn btn-secondary btn-small"
                    onClick={() => handlePlayAll(true)}
                  >
                    Shuffle
                  </button>
                </>
              )}
            </>
          )}
          {currentView.type === "playlist" && currentPlaylist && (
            <>
              <span className="playlist-title">
                {currentPlaylist.name}
              </span>
              <span className="playlist-filter-desc">
                {currentPlaylist.filters
                  .map((f) => formatFilter(f, workspaces, true))
                  .join(" & ")}
              </span>
              <select
                className="filter-select"
                value={currentPlaylist.playback_mode}
                onChange={(e) =>
                  handleSetPlaybackMode(currentPlaylist.id, e.target.value)
                }
              >
                <option value="sequential">Sequential</option>
                <option value="random">Random</option>
                <option value="repeat">Repeat</option>
              </select>
              <button
                className="btn btn-primary btn-small"
                onClick={() =>
                  handlePlayAll(currentPlaylist.playback_mode === "random")
                }
              >
                Play
              </button>
              <button
                className="btn btn-secondary btn-small"
                onClick={() => {
                  setEditingPlaylist(currentPlaylist);
                  setShowPlaylistDialog(true);
                }}
              >
                Edit
              </button>
            </>
          )}
        </div>

        {/* Video player - rendered outside normal flow to avoid re-renders */}
        {playingFile && (
          <MemoizedVideoPlayer
            file={playingFile}
            onClose={handleClosePlayer}
            onNext={playQueue.length > 1 ? handlePlayNext : undefined}
            onPrev={playQueue.length > 1 ? handlePlayPrev : undefined}
            queuePosition={playQueueIndex + 1}
            queueTotal={playQueue.length}
          />
        )}

        {mediaFiles.length === 0 ? (
          <div className="empty-state">
            {currentView.type === "playlist" ? (
              <>
                <h2>No matching files</h2>
                <p>Adjust the filter conditions for this playlist</p>
                <button
                  className="btn btn-primary"
                  onClick={() => {
                    setEditingPlaylist(currentPlaylist ?? undefined);
                    setShowPlaylistDialog(true);
                  }}
                >
                  Edit Playlist
                </button>
              </>
            ) : (
              <>
                <h2>No media files</h2>
                <p>Add a workspace to scan for video files</p>
                <button
                  className="btn btn-primary"
                  onClick={() => setShowWorkspaceDialog(true)}
                >
                  Add Workspace
                </button>
              </>
            )}
          </div>
        ) : (
          <div className="media-list" ref={mediaListRef}>
            <div className="media-item media-list-header" onClick={selectAll}>
              <input
                type="checkbox"
                checked={allSelected}
                onChange={selectAll}
                onClick={(e) => e.stopPropagation()}
              />
              <span className="filename">
                {allSelected ? "Deselect all" : "Select all"} (
                {mediaFiles.length})
              </span>
            </div>
            {mediaFiles.map((file) => (
              <div
                key={file.id}
                className={`media-item ${selectedFiles.has(file.id) ? "selected" : ""}`}
                onClick={() => toggleFileSelection(file.id)}
                onDoubleClick={() => handlePlayFile(file)}
              >
                <input
                  type="checkbox"
                  checked={selectedFiles.has(file.id)}
                  onChange={() => toggleFileSelection(file.id)}
                  onClick={(e) => e.stopPropagation()}
                />
                <span className="filename">{file.filename}</span>
                {file.series_name && (
                  <span className="meta">
                    {file.series_name}
                    {file.series_number != null && ` #${file.series_number}`}
                  </span>
                )}
                <span className="meta">{formatSize(file.size_bytes)}</span>
                <span className="extension">.{file.extension}</span>
              </div>
            ))}
          </div>
        )}

        <div className="status-bar">
          <span>
            {mediaFiles.length} files
            {selectedFiles.size > 0 && ` | ${selectedFiles.size} selected`}
          </span>
          <span>{statusMessage}</span>
        </div>
      </div>

      {/* Dialogs */}
      {showWorkspaceDialog && (
        <WorkspaceDialog
          onSubmit={handleAddWorkspace}
          onClose={() => setShowWorkspaceDialog(false)}
        />
      )}
      {showPlaylistDialog && (
        <PlaylistDialog
          workspaces={workspaces}
          existing={editingPlaylist}
          onSubmit={handleCreatePlaylist}
          onClose={() => {
            setShowPlaylistDialog(false);
            setEditingPlaylist(undefined);
          }}
        />
      )}
    </div>
  );
}

export default App;
