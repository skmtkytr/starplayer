# Star Player - Claude Code Instructions

## Architecture

- **Tauri v2** + React + TypeScript
- Video streaming: **local HTTP server** (tiny_http on 127.0.0.1, random port)
  - DO NOT use Tauri asset protocol (`convertFileSrc`) - it does not work for video playback
  - DO NOT use custom URI scheme protocols (`stream://`) - platform-specific issues on Windows
- Database: SQLite (rusqlite with bundled feature)
- Playlists: filter-based smart playlists (dynamic query, not manual file addition)

## Development

```bash
make dev        # Start dev server (auto-detects arch, reinstalls node_modules if needed)
make test       # Run all tests (Rust + TypeScript + vitest)
make build      # Production build for current platform
make reinstall  # Force reinstall node_modules for current arch
```

## Testing

- `make test` runs: cargo test + tsc --noEmit + vitest run
- Tests MUST pass before every commit/push
- URL generation, path encoding, platform-specific behavior must have unit tests
- Simulate user workflow (`make dev` on Mac/Windows) before pushing

## Cross-platform Notes

- `node_modules` is platform-specific (native bindings). `package-lock.json` is gitignored.
- Makefile's `ensure-node-modules` handles arch detection via stamp file
- Video URL: `http://127.0.0.1:<port>/stream/<encodeURIComponent(path)>` - same on all platforms
- UNC paths (`\\server\share`) are supported via urlencoding round-trip

## CI

- GitHub Actions: test (Linux) → build (Linux, macOS arm64, Windows) → release (on tag push)
- Tag `v*` triggers release with artifacts attached to GitHub Release
