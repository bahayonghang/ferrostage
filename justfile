# Windows recipes use Windows PowerShell 5.1; no interactive profile is needed.
set windows-shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command"]

# Default recipe. `just` with no arguments runs CI.
default: ci

# List commands and their descriptions without building or installing anything.
help:
    @echo "Usage: just <recipe> [args]. Inspect a command with: just --show <recipe>. Bare just runs ci."
    @just --list --unsorted

# --- Fast checks (default features, no native PDFium runtime required) ---

# Format, lint, test, and license check. Default features only.
ci: fmt-check clippy test deny

# Rewrite Rust sources with rustfmt.
fmt:
    cargo fmt --all

# Fail if rustfmt would change a file.
fmt-check:
    cargo fmt --all -- --check

# Lint the workspace. Warnings fail the build.
clippy:
    cargo clippy --workspace --all-targets -- -D warnings

# Run unit and integration tests.
test:
    cargo test --workspace --all-targets

# --- Native checks (Windows/MSVC; see docs/packaging.md for runtime setup) ---

# Test PDFium-backed crates; set FERROSTAGE_TEST_PDFIUM to the verified DLL first.
test-native:
    cargo test -p ferrostage-pdf -p ferrostage-render -p ferrostage-document --features pdfium --all-targets --locked

# Lint PDFium-backed crates with warnings as errors; does not run native tests.
clippy-native:
    cargo clippy -p ferrostage-pdf -p ferrostage-render -p ferrostage-document --features pdfium --all-targets --locked -- -D warnings

# Native compile/lint only; desktop acceptance is a separate gate.
check-shell:
    cargo build -p ferrostage-app --features gpui,pdfium --locked
    cargo clippy -p ferrostage-app -p ferrostage-ui --features gpui,pdfium --all-targets --locked -- -D warnings
    cargo test -p ferrostage-ui --features test-support --lib --locked

# Reject GPL/AGPL and yanked-crate policy from deny.toml.
deny:
    cargo deny check

# --- Desktop build/package/install (gpui,pdfium; optional media stays off) ---
# Uncached tools/runtimes require network access. Cargo's target directory is honored.

# Build release desktop app and verified payload; does not install or publish.
build:
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts/windows-distribution.ps1 -Action build

# Package versioned Setup, portable ZIP and SHA256 under the resolved target/dist; no publishing.
release:
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts/windows-distribution.ps1 -Action release

# Build first so Cargo can restore a collected target-cache directory before staging.
# Stage verified PDFium notices and pdfium.dll into target/debug, then run the shell.
# Build/stage and run the debug desktop app; forward args, e.g. just dev -- check-config <path>.
dev *args:
    cargo build -p ferrostage-app --features gpui,pdfium --locked
    powershell -NoProfile -File scripts/stage-pdfium.ps1 -Destination target/debug
    cargo run -p ferrostage-app --features gpui,pdfium --locked -- {{args}}

# Build/package and silently install for this user; close FerroStage first. Settings are preserved.
install:
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts/windows-distribution.ps1 -Action install -Silent

# Open the registered native uninstaller; removes owned files, preserves settings and user PDFs.
uninstall:
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts/windows-distribution.ps1 -Action uninstall

# Packaging regressions; -ArchiveOnly needs no build/download, -OsPolicyOnly needs NSIS, default needs a release. -Lifecycle changes this user's installation.
test-distribution *args:
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test-windows-distribution.ps1 {{args}}

# --- Optional media (separate from the default desktop package) ---

# Fetch and verify the selected GStreamer runtime and prepare import libraries.
media-runtime:
    powershell -NoProfile -File scripts/prepare-media-runtime.ps1

# Build and lint the isolated media helper; no desktop or audio playback.
media-helper:
    powershell -NoProfile -File scripts/media-native.ps1 -Action build
    powershell -NoProfile -File scripts/media-native.ps1 -Action clippy

# Requires verified FERROSTAGE_TEST_MEDIA_HELPER and FERROSTAGE_TEST_MEDIA_RUNTIME.
test-media:
    cargo build -p ferrostage-media --example ipc_stall --features native-tests --locked
    cargo test -p ferrostage-media --features native-tests --locked
