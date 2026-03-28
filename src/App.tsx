import { useState, useEffect, useCallback } from "react";
import type { Workspace, MediaFile, Playlist } from "./types";
import {
  listWorkspaces,
  addWorkspace,
  removeWorkspace,
  scanWorkspace,
  listMediaFiles,
  listPlaylists,
  createPlaylist,
  deletePlaylist,
  addToPlaylist,
  getPlaylistItems,
  playFile,
  playPlaylist,
  setPlaybackMode,
} from "./hooks/useApi";
import { WorkspaceDialog } from "./components/WorkspaceDialog";
import { PlaylistDialog } from "./components/PlaylistDialog";

type View = { type: "library"; workspaceId?: string } | { type: "playlist"; playlistId: string };

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
  const [statusMessage, setStatusMessage] = useState("");

  const loadWorkspaces = useCallback(async () => {
    try {
      const ws = await listWorkspaces();
      setWorkspaces(ws);
    } catch (e) {
      console.error("Failed to load workspaces:", e);
    }
  }, []);

  const loadPlaylists = useCallback(async () => {
    try {
      const pl = await listPlaylists();
      setPlaylists(pl);
    } catch (e) {
      console.error("Failed to load playlists:", e);
    }
  }, []);

  const loadMediaFiles = useCallback(async () => {
    try {
      if (currentView.type === "playlist") {
        const items = await getPlaylistItems(currentView.playlistId);
        setMediaFiles(items);
      } else {
        const files = await listMediaFiles({
          workspaceId: currentView.workspaceId,
          search: searchQuery || undefined,
          extension: extensionFilter || undefined,
        });
        setMediaFiles(files);
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

  const handleCreatePlaylist = async (name: string) => {
    try {
      await createPlaylist(name);
      await loadPlaylists();
      setShowPlaylistDialog(false);
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

  const handleAddToPlaylist = async (playlistId: string) => {
    try {
      const ids = Array.from(selectedFiles);
      if (ids.length === 0) return;
      await addToPlaylist(playlistId, ids);
      setStatusMessage(`Added ${ids.length} files to playlist`);
      setSelectedFiles(new Set());
      await loadPlaylists();
      if (currentView.type === "playlist" && currentView.playlistId === playlistId) {
        await loadMediaFiles();
      }
    } catch (e) {
      setStatusMessage(`Error: ${e}`);
    }
  };

  const handlePlayFile = async (path: string) => {
    try {
      await playFile(path);
    } catch (e) {
      setStatusMessage(`Error: ${e}`);
    }
  };

  const handlePlayPlaylist = async (playlistId: string) => {
    try {
      await playPlaylist(playlistId);
    } catch (e) {
      setStatusMessage(`Error: ${e}`);
    }
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

  const formatSize = (bytes: number) => {
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
  };

  const extensions = [...new Set(mediaFiles.map((f) => f.extension))].sort();
  const currentPlaylist =
    currentView.type === "playlist"
      ? playlists.find((p) => p.id === currentView.playlistId)
      : null;

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
            <span className="count">{mediaFiles.length}</span>
          </div>
        </div>

        <div className="sidebar-section">
          <h3>Workspaces</h3>
          {workspaces.map((ws) => (
            <div
              key={ws.id}
              className={`sidebar-item ${currentView.type === "library" && currentView.workspaceId === ws.id ? "active" : ""}`}
              onClick={() => setCurrentView({ type: "library", workspaceId: ws.id })}
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
            className="sidebar-item"
            onClick={() => setShowWorkspaceDialog(true)}
            style={{ color: "var(--accent)" }}
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
              onClick={() => setCurrentView({ type: "playlist", playlistId: pl.id })}
              onContextMenu={(e) => {
                e.preventDefault();
                if (confirm(`Delete playlist "${pl.name}"?`)) {
                  handleDeletePlaylist(pl.id);
                }
              }}
            >
              {pl.name}
              <span className="count">{pl.item_count}</span>
            </div>
          ))}
          <div
            className="sidebar-item"
            onClick={() => setShowPlaylistDialog(true)}
            style={{ color: "var(--accent)" }}
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
            </>
          )}
          {currentView.type === "playlist" && currentPlaylist && (
            <>
              <span style={{ fontSize: "14px", fontWeight: 600 }}>
                {currentPlaylist.name}
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
                onClick={() => handlePlayPlaylist(currentPlaylist.id)}
              >
                Play
              </button>
            </>
          )}
          {selectedFiles.size > 0 && playlists.length > 0 && (
            <select
              className="filter-select"
              value=""
              onChange={(e) => {
                if (e.target.value) handleAddToPlaylist(e.target.value);
              }}
            >
              <option value="">
                Add {selectedFiles.size} to playlist...
              </option>
              {playlists.map((pl) => (
                <option key={pl.id} value={pl.id}>
                  {pl.name}
                </option>
              ))}
            </select>
          )}
        </div>

        {mediaFiles.length === 0 ? (
          <div className="empty-state">
            <h2>No media files</h2>
            <p>Add a workspace to scan for video files</p>
            <button
              className="btn btn-primary"
              onClick={() => setShowWorkspaceDialog(true)}
            >
              Add Workspace
            </button>
          </div>
        ) : (
          <div className="media-list">
            {mediaFiles.map((file) => (
              <div
                key={file.id}
                className={`media-item ${selectedFiles.has(file.id) ? "selected" : ""}`}
                onClick={() => toggleFileSelection(file.id)}
                onDoubleClick={() => handlePlayFile(file.path)}
              >
                <input
                  type="checkbox"
                  checked={selectedFiles.has(file.id)}
                  onChange={() => toggleFileSelection(file.id)}
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
          onSubmit={handleCreatePlaylist}
          onClose={() => setShowPlaylistDialog(false)}
        />
      )}
    </div>
  );
}

export default App;
