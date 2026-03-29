# Star Player - Claude Code Instructions

## Architecture

- **Tauri v2** + React + TypeScript
- Video playback: **mpv** bundled as Tauri sidecar (`src-tauri/binaries/mpv-<target-triple>`)
  - DO NOT use HTML5 `<video>` element - codec support is too limited, causes flickering
  - DO NOT use Tauri asset protocol (`convertFileSrc`) - does not work for video playback
  - DO NOT use custom HTTP server for streaming - causes choppy playback
- Database: SQLite (rusqlite with bundled feature)
- Playlists: filter-based smart playlists (dynamic query, not manual file addition)

## Development

```bash
make dev        # Download mpv + start dev server (auto-detects arch)
make test       # Run all tests (Rust + TypeScript + vitest)
make build      # Production build for current platform
make reinstall  # Force reinstall node_modules for current arch
```

`make dev` automatically:
1. Checks node_modules arch stamp, reinstalls if needed
2. Downloads mpv binary for current platform if not present
3. Starts Tauri dev server

## Testing

- `make test` runs: cargo test + tsc --noEmit + vitest run
- Tests MUST pass before every commit/push
- Simulate user workflow (`make dev` on Mac/Windows) before pushing

## Cross-platform Notes

- `node_modules` is platform-specific. `package-lock.json` is gitignored.
- `src-tauri/binaries/` is gitignored. mpv binaries are downloaded per-platform.
- Makefile's `ensure-node-modules` handles arch detection via stamp file
- Makefile's `ensure-mpv` downloads correct mpv binary for current platform

## CI

- GitHub Actions: test (Linux) → build (Linux, macOS arm64, Windows) → release (on tag push)
- Each build job downloads platform-specific mpv binary before `tauri build`
- Tag `v*` triggers release with artifacts attached to GitHub Release
