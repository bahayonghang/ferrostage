# FerroStage

[中文](README.zh-CN.md) | English

A modern, GPU-accelerated, lock-free PDF & Beamer presentation stage built with Rust and GPUI.

Windows 11 is the primary target. The application operates offline: no telemetry, no network calls, and no accounts.

The optional desktop build links the presenter workbench, single-worker PDFium rasterization, complete settings, external PDF hot reload, animated GIF/WebP playback, and isolated native video decoding. Default workspace checks stay independent of native SDKs. Implementation and scoped verification status are tracked in [feature parity](docs/feature-parity.md).

## Features

- **Document Management**: Open PDFs through CLI arguments, file dialogs, drag-and-drop, or the recent files list. Files open in `Ready` state; presentation starts explicitly.
- **Dual-Window Presentation**: Dedicated Presenter Workbench and Audience Projection windows. Projection follows monitor roles (`Presenter`, `Audience`, `Off`), with screen identification overlays, topology views, one-key display swap, and click-to-advance on the audience window.
- **Beamer Integration**: Automatic detection of same-page left/right or top/bottom notes, layout navigation menu with manual overrides (`None`, `Left`, `Right`, `Top`, `Bottom`), dedicated `notes:` slide pairing, `\pause` overlay steps (`SlideMap`), and `PageLabel` logical page numbers.
- **Presenter Workbench**: Left-hand current slide view, scrollable and zoomable private speaker notes above the next slide preview, elapsed timer, countdown timer with warning alerts, wall clock, page jump input, and an on-demand 1–16 page thumbnail grid (1–4 columns/rows).
- **In-Memory Annotations & Pointer**: Normalized vector laser pointer and freehand digital pen, highlighter, eraser, and clear actions. All strokes stay in session memory and synchronize in real time to the audience screen; the PDF file is never modified.
- **External PDF Hot Reload**: Monitors external file changes with `notify`, applies 300 ms debounce, checks trailing `%%EOF`, and executes an atomic background swap that preserves the current slide, presentation stage, and ink annotations.
- **Media Playback**: Embedded pure-Rust GIF and animated WebP playback, plus an isolated out-of-process GStreamer native video helper (BGRA offscreen frames, zero child HWNDs, graceful fallback on missing dependencies).
- **Settings & Visual Themes**: Full TOML configuration backend with GUI settings and a `check-config` CLI validator. Dark, light, and system themes with Claude-inspired visual tokens.

## GitHub topics

Suggested topics keep `beamer` and drop `latex` when that topic is set. `beamer` means consuming a Beamer-exported PDF, not a LaTeX toolchain.

## Workspace

| Crate | Role |
| --- | --- |
| `ferrostage-app` | Binary crate and CLI coordinator (`ferrostage`). |
| `ferrostage-ui` | GPUI presentation windows and settings views (Zed `v1.23.3` maintained tree). |
| `ferrostage-core` | State machine, commands, geometry, and presentation lifecycle without UI or PDF dependencies. |
| `ferrostage-document` | Document model, slide mapping, Beamer layout detection, and speaker notes. |
| `ferrostage-pdf` | Read-only PDF parsing (`lopdf`) and dynamic PDFium rasterization. |
| `ferrostage-render` | Single render worker, tile scheduler, and bounded L1/L2/L3 CPU/GPU cache budgets. |
| `ferrostage-media` | Pure-Rust GIF/WebP decoder, shared BGRA frames, and helper protocol supervisor. |
| `ferrostage-media-helper` | Isolated out-of-process GStreamer video decoder; zero child window handles. |
| `ferrostage-platform` | Monitor topology enumeration, display identification, and `notify`-based file reload watcher. |
| `ferrostage-settings` | TOML settings schema, storage, validation, and migration. |

## Commands

```text
just ci       # fmt --check, clippy -D warnings, test, cargo deny; default features
just dev      # stage verified PDFium into target/debug, then run with gpui,pdfium
just build    # release gpui,pdfium app; double-click target\release\ferrostage.exe
just install  # install gpui,pdfium and stage pdfium.dll into Cargo's bin
```

Toolchain: Rust 1.99.0, edition 2024 (`rust-toolchain.toml`).

`just ci` stays on default features. `just dev`, `just build`, and `just install` enable `gpui` and `pdfium` and stage the hash-checked `pdfium.dll` from [packaging](docs/packaging.md). `just dev` and `just build` also stage `LICENSE` and `licenses/`. `just install` stages only the DLL, leaving the notices with `target\release`. The application never downloads PDFium and never writes the user's PDF.

The double-click artifact is `target\release\ferrostage.exe` after `just build`. That recipe links the featured app last. Video playback requires the separate media helper and runtime, built with `cargo build -p ferrostage-app --features gpui,pdfium,media --locked`.

`just dev` keeps a console. The release executable uses the Windows GUI subsystem, so Explorer opens `target\release\ferrostage.exe` without a console. `just dev -- check-config <path>` prints the read-only settings report in the terminal. An interactive release `check-config` has no console. Redirected stdout from release `check-config` still prints the result.

## License

MIT OR Apache-2.0. See [LICENSE](LICENSE).

Pympress is GPL-2.0-or-later. FerroStage does not copy or translate that source. `cargo deny` rejects GPL and AGPL, which also rules out MuPDF.

## Docs

- [docs/PROJECT_PLAN.md](docs/PROJECT_PLAN.md) — architecture and milestones
- [docs/architecture.md](docs/architecture.md)
- [docs/feature-parity.md](docs/feature-parity.md) — feature implementation and verification status
- [docs/packaging.md](docs/packaging.md) — PDFium and packaging notes
- [AGENTS.md](AGENTS.md) — agent and contributor rules
