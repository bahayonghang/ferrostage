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

Do not add GPL or AGPL crates. `deny.toml` enforces this. Do not add MuPDF. ADR-005/008 approve only three exact additional license exceptions for task07; do not broaden the global allowlist or advisory policy. The full product graph uses `cargo deny --all-features --locked --offline check` as an additional native dependency gate; candidate-only success is insufficient.

## Tests

- State transitions live next to `PresentationState::apply_at`.
- Property tests use `proptest` for invariants that are true for a range (generation ordering).
- Do not `#[ignore]` a test to make CI green.
- Default workspace checks exercise the pure Rust path. The separate `pdfium` feature suite requires the pinned native runtime and must fail when it is missing. A separate integration-test process checks failed initialization before any process-global bindings exist.
- Raster goldens assert BGRA ordering, rotated/CropBox coordinates, alpha and same-density crop pixels. Performance uses uncached real renders, 20 warmups and all 200 timed samples; record runtime/fixture/font hashes and keep queue/upload time separate. Mock results do not establish native or GPU performance.
- Repeated native monitor enumeration must release every acquired device context on success and failure. Do not infer this from a guard's type name: winsafe0.0.29 skips ReleaseDC for a NULL HWND owner. A replacement acquisition must preserve complete per-monitor identity/geometry/scale provenance and pass a real Windows GDI-resource-count regression; desktop-DPI contexts and full monitor coverage also need native application verification. Do not hide resource exhaustion by reusing stale topology.
- Beamer notes acceptance uses a real TeX-compiled nonempty PDF with distinct public slide/private note canaries, physical-page labels and overlay expectations. Verify the original compiled page labels rather than assuming visible frame footers imply correct PDF labels. Independently establish left/right crop content, real PDFium rendering, presenter notes display and audience isolation. A placeholder or document-only result cannot prove UI notes availability.

## Forbidden

- Importing GPUI, `winit`, or `windows` into `ferrostage-core`.
- A child `HWND` for video. Frames are `SharedFrame` (BGRA8).
- Converting BGRA to RGBA on the audience path.
- Copying Pympress (GPL) source into this repository.
- Linking PDFium or Zed in the default feature set until a task is ready to own that build time.
