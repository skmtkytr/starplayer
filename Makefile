# Star Player - Cross-platform build system
# Usage:
#   make setup  # Install dependencies for current platform
#   make dev    # Run dev server (auto-reinstalls if arch changed)
#   make build  # Production build
#   make test   # Run all tests
#   make clean  # Clean build artifacts

# --- Platform detection ---
UNAME_S := $(shell uname -s)
UNAME_M := $(shell uname -m)

ifeq ($(UNAME_S),Darwin)
  PLATFORM := macos
  ifeq ($(UNAME_M),arm64)
    ARCH := aarch64-apple-darwin
    ARCH_SHORT := darwin-arm64
  else
    ARCH := x86_64-apple-darwin
    ARCH_SHORT := darwin-x64
  endif
  SYSTEM_DEPS_CMD := @echo "macOS: ensure Xcode CLT is installed (xcode-select --install)"
else ifeq ($(UNAME_S),Linux)
  PLATFORM := linux
  ifeq ($(UNAME_M),aarch64)
    ARCH := aarch64-unknown-linux-gnu
    ARCH_SHORT := linux-arm64
  else
    ARCH := x86_64-unknown-linux-gnu
    ARCH_SHORT := linux-x64
  endif
  SYSTEM_DEPS_CMD := sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.1-dev \
    libappindicator3-dev librsvg2-dev patchelf libssl-dev
else
  # Windows (MSYS2/Git Bash)
  PLATFORM := windows
  ARCH := x86_64-pc-windows-msvc
  ARCH_SHORT := win-x64
  SYSTEM_DEPS_CMD := @echo "Windows: ensure WebView2 runtime is installed"
endif

# Stamp file to track which arch node_modules was installed for
NODE_STAMP := node_modules/.arch-stamp
CURRENT_STAMP := $(shell cat $(NODE_STAMP) 2>/dev/null)

# --- Commands ---
NPM := npm
CARGO := cargo
TAURI := npx tauri

MPV_VERSION := 0.41.0
MPV_BIN := src-tauri/binaries/mpv-$(ARCH)
ifeq ($(UNAME_S),Darwin)
  ifeq ($(UNAME_M),arm64)
    MPV_URL := https://github.com/mpv-player/mpv/releases/download/v$(MPV_VERSION)/mpv-v$(MPV_VERSION)-macos-15-arm.zip
  else
    MPV_URL := https://github.com/mpv-player/mpv/releases/download/v$(MPV_VERSION)/mpv-v$(MPV_VERSION)-macos-15-x86_64.zip
  endif
else ifeq ($(UNAME_S),Linux)
  MPV_URL :=
else
  MPV_URL := https://github.com/mpv-player/mpv/releases/download/v$(MPV_VERSION)/mpv-v$(MPV_VERSION)-x86_64-pc-windows-msvc.zip
endif

.PHONY: help setup setup-system setup-rust setup-node dev build test \
        test-rust test-frontend check lint clean reinstall info \
        ensure-node-modules ensure-mpv

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

info: ## Show detected platform info
	@echo "Platform:    $(PLATFORM)"
	@echo "Arch:        $(ARCH_SHORT)"
	@echo "Target:      $(ARCH)"
	@echo "node_modules: $(if $(CURRENT_STAMP),$(CURRENT_STAMP),not installed)"
	@echo "Node:        $(shell node --version 2>/dev/null || echo 'not found')"
	@echo "Rust:        $(shell rustc --version 2>/dev/null || echo 'not found')"
	@echo "npm:         $(shell npm --version 2>/dev/null || echo 'not found')"
	@echo "cargo:       $(shell cargo --version 2>/dev/null || echo 'not found')"

# --- Arch-aware node_modules ---

ensure-node-modules:
	@if [ ! -d node_modules ] || [ "$(CURRENT_STAMP)" != "$(ARCH_SHORT)" ]; then \
		echo "Installing node_modules for $(ARCH_SHORT)..."; \
		rm -rf node_modules package-lock.json; \
		$(NPM) install; \
		echo "$(ARCH_SHORT)" > $(NODE_STAMP); \
	fi

# --- Setup ---

setup: setup-system setup-rust ensure-node-modules ## Full setup for current platform
	@echo "Setup complete for $(PLATFORM) ($(ARCH_SHORT))"

setup-system: ## Install system-level dependencies
	$(SYSTEM_DEPS_CMD)

setup-rust: ## Ensure Rust toolchain is ready
	rustup default stable
	rustup target add $(ARCH)

# --- mpv sidecar ---

ensure-mpv:
	@if [ ! -f "$(MPV_BIN)" ] && [ ! -f "$(MPV_BIN).exe" ]; then \
		echo "Downloading mpv for $(ARCH)..."; \
		mkdir -p src-tauri/binaries; \
		if [ "$(UNAME_S)" = "Darwin" ]; then \
			curl -L "$(MPV_URL)" -o /tmp/mpv-download.zip; \
			rm -rf /tmp/mpv-extract; \
			cd /tmp && unzip -o mpv-download.zip -d mpv-extract; \
			if ls /tmp/mpv-extract/*.tar.gz 1>/dev/null 2>&1; then \
				cd /tmp/mpv-extract && tar xzf *.tar.gz; \
			fi; \
			if [ -d /tmp/mpv-extract/mpv.app ]; then \
				rm -rf "$(CURDIR)/src-tauri/binaries/mpv.app"; \
				cp -R /tmp/mpv-extract/mpv.app "$(CURDIR)/src-tauri/binaries/mpv.app"; \
				ln -sf mpv.app/Contents/MacOS/mpv "$(CURDIR)/$(MPV_BIN)"; \
			else \
				echo "ERROR: mpv.app not found in archive"; \
				find /tmp/mpv-extract -type f | head -10; \
				exit 1; \
			fi; \
			rm -rf /tmp/mpv-download.zip /tmp/mpv-extract; \
		elif [ "$(PLATFORM)" = "windows" ]; then \
			curl -L "$(MPV_URL)" -o /tmp/mpv-download.zip; \
			rm -rf /tmp/mpv-extract; \
			cd /tmp && unzip -o mpv-download.zip -d mpv-extract; \
			cp /tmp/mpv-extract/mpv.exe "$(CURDIR)/$(MPV_BIN).exe"; \
			rm -rf /tmp/mpv-download.zip /tmp/mpv-extract; \
		else \
			echo "Linux: install mpv via package manager (apt install mpv)"; \
		fi; \
	fi

# --- Development ---

dev: ensure-node-modules ensure-mpv ## Start Tauri dev server
	$(TAURI) dev

dev-frontend: ensure-node-modules ## Start frontend dev server only (no Tauri)
	$(NPM) run dev

# --- Build ---

build: ensure-node-modules ensure-mpv ## Production build for current platform
	$(TAURI) build --target $(ARCH)

build-frontend: ensure-node-modules ## Build frontend only
	$(NPM) run build

build-rust: ## Build Rust backend only
	cd src-tauri && $(CARGO) build --release --target $(ARCH)

build-debug: ensure-node-modules ## Debug build for current platform
	$(TAURI) build --debug --target $(ARCH)

# --- Test ---

test: test-rust test-frontend ## Run all tests
	@echo "All tests passed"

test-rust: ## Run Rust tests
	cd src-tauri && $(CARGO) test

test-frontend: ensure-node-modules ## Run frontend type check and unit tests
	npx tsc --noEmit
	npx vitest run --passWithNoTests

# --- Quality ---

check: ensure-node-modules ## Run cargo check + tsc
	cd src-tauri && $(CARGO) check
	npx tsc --noEmit

lint: ensure-node-modules ## Run clippy + tsc strict
	cd src-tauri && $(CARGO) clippy -- -D warnings
	npx tsc --noEmit

# --- Clean ---

clean: ## Remove build artifacts
	rm -rf dist
	cd src-tauri && $(CARGO) clean

reinstall: ## Force reinstall node_modules for current arch
	rm -rf node_modules package-lock.json
	$(NPM) install
	@echo "$(ARCH_SHORT)" > $(NODE_STAMP)
	@echo "Reinstalled for $(PLATFORM) ($(ARCH_SHORT))"
