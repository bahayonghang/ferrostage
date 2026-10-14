# FerroStage project plan

Status: workspace skeleton (2026-10-14). The presenter is not implemented yet.

FerroStage is a clean-room PDF and Beamer presentation stage. The behavior target is the dual-screen presenter workflow (current slide, next slide, notes, laser, pen, blank, media, hot reload). Pympress source is not an input. The license is MIT OR Apache-2.0.

## Decisions

- Rust edition 2024. Toolchain pin: 1.99.0 (latest stable installed for this repo; older 1.88–1.92 baselines are not used).
- UI: GPUI from Zed tag `v1.23.3` (published 2026-10-12), or `gpui-kit` / `gpui-component` 0.7.3 which vendors `gpui-pre` 0.3.9. Do not depend on crates.io `gpui` 0.2.x. The skeleton does not link GPUI, so `cargo test` does not clone Zed.
- PDF raster: `pdfium-render` 0.9.4, `PdfBitmapFormat::BGRA`, memory load, one dedicated render thread. Not linked yet.
- PDF structure: `lopdf` 0.45.0 is linked for a parse check. `hayro-syntax` 0.8.0 is the planned second reader and is not linked yet.
- Media: offscreen BGRA frames shared as one `Arc`. No child `HWND`.
- Hot reload: `notify` 8.2.0 (9.0.0 is still a release candidate), 300 ms debounce, wait for `%%EOF`, shadow document, bump `DocumentGeneration`. Never truncate the user's PDF in place. Writes go to `*.ferrostage.tmp`, then an atomic rename.
- Cache budgets: L1 GPU 256 MiB (current ±2), L2 CPU 512 MiB (current ±6), L3 thumbnails 64 MiB.
- First OS: Windows 11 (DX11). Ubuntu 24.04 and macOS 15 come later. No telemetry.

## Crate map

See the root `README.md`. `ferrostage-core` owns the state machine and must not import GPUI. The audience window may render only `AudienceProjectionSnapshot`. That type has no speaker-notes field.

## Dual-screen invariants

1. Audience pixels come from a snapshot, not from a shared mutable document.
2. The snapshot generation matches the document generation the presenter committed.
3. Speaker notes are presenter-only.
4. Blank mode is part of the snapshot, so both windows agree.
5. Swapping screens exchanges window roles. It does not change the page or the crop.
6. A shadow document is published only with a strictly newer generation. Stale renders are dropped.

Beamer overlay steps are not modeled yet. `NextLogicalSlide` currently moves one physical page.

## Milestones

| Phase | Outcome | State |
| --- | --- | --- |
| Bootstrap | Workspace, toolchain, `just ci`, specs, license gate | This tree |
| PoC | GPUI window, PDFium BGRA page, sub-rect crop, offscreen frame, lock-free reload | Not started |
| Presenter | Dual window, notes, laser, pen, blank, hot reload | Not started |
| Media and packaging | GIF/video, installers | Not started |

## Out of scope for the skeleton

Native PDFium binaries, libVLC, GStreamer, URW Base35 font files, and the Zed source tree. Fonts, if added later, go under `assets/fonts/urw-base35/` with their own license. Do not vendor them until that license is checked.
