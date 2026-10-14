# Quality Guidelines

## Gate

`just ci` is the definition of done for a change:

1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test --workspace --all-targets`
4. `cargo deny check`

Rust 1.99.0, edition 2024. New crates inherit workspace lints (`unsafe_code = "forbid"`, `clippy::all = "warn"`).

## Dependency pins

Workspace `[workspace.dependencies]` records the latest stable release we have checked (2026-10-14): `thiserror` 2.0.21, `tracing` 0.1.44, `tracing-subscriber` 0.3.23, `clap` 4.6.7, `serde` 1.0.229, `toml` 1.1.8, `lopdf` 0.45.0, `notify` 8.2.0, `proptest` 1.12.0. Bump a pin by editing that table, not by writing a version inside one crate.

Do not add GPL or AGPL crates. `deny.toml` enforces this. Do not add MuPDF.

## Tests

- State transitions live next to `PresentationState::apply`.
- Property tests use `proptest` for invariants that are true for a range (generation ordering).
- Do not `#[ignore]` a test to make CI green.

## Forbidden

- Importing GPUI, `winit`, or `windows` into `ferrostage-core`.
- A child `HWND` for video. Frames are `SharedFrame` (BGRA8).
- Converting BGRA to RGBA on the audience path.
- Copying Pympress (GPL) source into this repository.
- Linking PDFium or Zed in the default feature set until a task is ready to own that build time.
