set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# Default recipe. `just` with no arguments runs CI.
default: ci

# Format, lint, test, and license check.
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

# Reject GPL/AGPL and yanked-crate policy from deny.toml.
deny:
    cargo deny check

# Release build of every crate.
build:
    cargo build --workspace --release

# Run the FerroStage binary. The presenter window is not linked yet.
dev:
    cargo run -p ferrostage-app

# Install the ferrostage binary into Cargo's bin directory.
install:
    cargo install --path crates/ferrostage-app --locked
