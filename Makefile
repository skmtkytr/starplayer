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

.PHONY: help setup setup-system setup-rust setup-node dev build test \
        test-rust test-frontend check lint clean reinstall info \
        ensure-node-modules

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
ifneq ($(CURRENT_STAMP),$(ARCH_SHORT))
	@echo "Arch changed ($(or $(CURRENT_STAMP),none) -> $(ARCH_SHORT)), reinstalling node_modules..."
	rm -rf node_modules package-lock.json
	$(NPM) install
	@echo "$(ARCH_SHORT)" > $(NODE_STAMP)
else
	@# node_modules is up to date for this arch
endif

# Also handle package.json changes
$(NODE_STAMP): package.json
	@echo "package.json changed, reinstalling node_modules..."
	rm -rf node_modules package-lock.json
	$(NPM) install
	@echo "$(ARCH_SHORT)" > $(NODE_STAMP)

# --- Setup ---

setup: setup-system setup-rust ensure-node-modules ## Full setup for current platform
	@echo "Setup complete for $(PLATFORM) ($(ARCH_SHORT))"

setup-system: ## Install system-level dependencies
	$(SYSTEM_DEPS_CMD)

setup-rust: ## Ensure Rust toolchain is ready
	rustup default stable
	rustup target add $(ARCH)

# --- Development ---

dev: ensure-node-modules ## Start Tauri dev server
	$(TAURI) dev

dev-frontend: ensure-node-modules ## Start frontend dev server only (no Tauri)
	$(NPM) run dev

# --- Build ---

build: ensure-node-modules ## Production build for current platform
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

test-frontend: ensure-node-modules ## Run frontend type check
	npx tsc --noEmit

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
