# Star Player

Cross-platform media library manager with smart playlists and VLC integration.

Built with Tauri v2, React, TypeScript, and SQLite.

## Features

- **Workspace-based library** - Scan directories for video files and auto-index metadata
- **Smart playlists** - Filter-based dynamic playlists with multiple conditions (contains, starts_with, equals, etc.)
- **Series detection** - Automatically extracts series name and episode number from filenames (`S01E05`, `EP12`, `#03`, `- 01` patterns)
- **Playback modes** - Sequential, Random, or Repeat per playlist
- **VLC integration** - Single file or queued multi-file playback via VLC
- **Cross-platform** - Linux, macOS (Apple Silicon / Intel), Windows

## Screenshots

<!-- TODO: Add screenshots -->

## Requirements

- [Node.js](https://nodejs.org/) 22+
- [Rust](https://rustup.rs/) stable
- [VLC](https://www.videolan.org/) 3.x
- Linux: `libgtk-3-dev libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libssl-dev`

## Development

```bash
make setup    # Install all dependencies
make dev      # Start dev server (downloads VLC + launches Tauri)
make test     # Run all tests (cargo test + tsc + vitest)
make build    # Production build for current platform
```

See `make help` for all available targets.

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Framework | Tauri v2 |
| Frontend | React 19, TypeScript, Vite |
| Backend | Rust, SQLite (rusqlite) |
| Playback | VLC (external process) |
| CI/CD | GitHub Actions |

## Architecture

```
src/                  # React frontend
  App.tsx             # Main UI (sidebar, media list, search, filters)
  components/         # PlaylistDialog, WorkspaceDialog
  hooks/              # useApi (Tauri command bridge)
  types/              # TypeScript interfaces

src-tauri/src/        # Rust backend
  lib.rs              # Tauri app setup + command registration
  commands.rs         # 14 Tauri commands (workspace, media, playlist, playback)
  db.rs               # SQLite operations + dynamic query builder
  models.rs           # Data models (Workspace, MediaFile, Playlist, PlaylistFilter)
  scanner.rs          # Directory traversal + series name extraction
```

## CI/CD

Pushes to `master` trigger:

1. **Test** (Linux) - lint, type check, unit tests, clippy
2. **Build** (Linux / macOS arm64 / Windows) - platform builds with bundled VLC
3. **Release** (on `v*` tag) - artifacts uploaded to GitHub Releases

## License

<!-- TODO: Add license -->
