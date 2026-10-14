# Quality Guidelines

UI changes still go through `just ci`. The view crate does not get a weaker clippy profile.

## Window rules

- Audience view input type is `AudienceProjectionSnapshot`.
- Presenter view may show `SlideNotes`.
- Both views share one `Arc` frame from `ferrostage-media`. No child `HWND` (Airspace).
- Pixel format stays BGRA8.

## GPUI pin

```rust
pub const ZED_GPUI_TAG: &str = "v1.23.3";
pub const GPUI_KIT_VERSION: &str = "0.7.3";
```

A change of either constant updates `docs/adr/ADR-001-gpui-from-zed-tag.md` in the same commit.

## Forbidden

- Linking GPUI from `ferrostage-core`.
- A web frontend, a localhost control page, or telemetry to "measure frame time".
- Blocking the GPUI thread on PDFium. Raster requests go to the single render worker.
- `unwrap` on a missing monitor. Fall back to the primary display and log at `warn`.
