set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# Default recipe. `just` with no arguments runs CI.
default: ci

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

# Requires FERROSTAGE_TEST_PDFIUM pointing to the verified native library file.
test-native:
    cargo test -p ferrostage-pdf -p ferrostage-render -p ferrostage-document --features pdfium --all-targets --locked

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

# Release desktop build. The gpui,pdfium app link is the last cargo step,
# then the verified PDFium runtime is staged beside target/release/ferrostage.exe.
build:
    cargo build --workspace --release --locked --exclude ferrostage-app
    cargo build -p ferrostage-app --release --features gpui,pdfium --locked
    powershell -NoProfile -File scripts/stage-pdfium.ps1 -Destination target/release

# Build first so Cargo can restore a collected target-cache directory before staging.
# Stage verified PDFium notices and pdfium.dll into target/debug, then run the shell.
# Extra arguments are forwarded (`just dev -- check-config <path>`).
dev *args:
    cargo build -p ferrostage-app --features gpui,pdfium --locked
    powershell -NoProfile -File scripts/stage-pdfium.ps1 -Destination target/debug
    cargo run -p ferrostage-app --features gpui,pdfium --locked -- {{args}}

# Install the gpui,pdfium binary and stage only pdfium.dll beside it.
install:
    cargo install --path crates/ferrostage-app --locked --features gpui,pdfium
    powershell -NoProfile -File scripts/stage-pdfium.ps1 -DllOnly -CargoBin

# Selected GStreamer runtime preparation and isolated helper (no desktop/audio).
media-runtime:
    powershell -NoProfile -File scripts/prepare-media-runtime.ps1

media-helper:
    powershell -NoProfile -File scripts/media-native.ps1 -Action build
    powershell -NoProfile -File scripts/media-native.ps1 -Action clippy

# Requires verified FERROSTAGE_TEST_MEDIA_HELPER and FERROSTAGE_TEST_MEDIA_RUNTIME.
test-media:
    cargo build -p ferrostage-media --example ipc_stall --features native-tests --locked
    cargo test -p ferrostage-media --features native-tests --locked
