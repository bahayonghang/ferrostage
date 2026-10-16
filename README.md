# FerroStage

A modern, GPU-accelerated, lock-free PDF & Beamer presentation stage built with Rust and GPUI.

Windows 11 is the first target. The process stays offline: no telemetry, no account.

The optional desktop build includes the presenter workbench, single-worker PDFium rendering, complete settings, external PDF hot reload, GIF/WebP and isolated native media playback. Default workspace checks remain independent of native SDKs. Implementation and scoped verification status are tracked in [feature parity](docs/feature-parity.md); full hardware, visual-matrix and performance qualification is separate.

## Planned for v1.0

- Open a PDF (CLI, dialog, drag-and-drop, recent files). Import lands in Ready; the speaker starts explicitly.
- Separate presenter and audience windows, projection by monitor role, one-key swap, black/white blank, fullscreen.
- Beamer: same-page left/right or top/bottom notes, `notes:` pages, `\pause` overlay steps, PageLabel logical pages.
- Presenter workbench: current page, next page, notes, timer (count up, count down, warning), clock, jump preview, 1–16 page thumbnail grid.
- Normalized laser pointer and ink stored in session memory. Nothing is written to the PDF.
- Settings system and settings UI; monitor identification and role assignment.
- Generic hot reload when the PDF file on disk is replaced (P2).
- GIF and offscreen video (P2).

## Not in v1.0

A LaTeX source or compiler toolchain; PDF annotation editing or write-back (read-only discovery of media structure stays); rehearsal-statistics export; Beamer `animate`; cloud, accounts, telemetry; PPTX.

## GitHub topics

Suggested topics keep `beamer` and drop `latex` when that topic is set. `beamer` means consuming a Beamer-exported PDF, not a LaTeX toolchain.

## Workspace

| Crate | Role |
| --- | --- |
| `ferrostage-app` | Binary. `just dev` runs it. |
| `ferrostage-ui` | GPUI presenter and snapshot-only Audience; maintained Zed `v1.23.3` source, see ADR-008. |
| `ferrostage-core` | Presentation state machine. No GPUI and no PDF. |
| `ferrostage-document` | Pages, speaker notes, media slots. Notes never enter the audience snapshot. |
| `ferrostage-pdf` | Read-only `lopdf` 0.45 structure and optional `pdfium-render` 0.9.4 adapter. |
| `ferrostage-render` | Single render worker and L1/L2/L3 budgets. |
| `ferrostage-media` | Pure GIF/WebP decoder, bounded helper protocol and single-owner supervision. |
| `ferrostage-media-helper` | Optional native GStreamer process; offscreen BGRA, no child window handles. |
| `ferrostage-platform` | Monitor identity/topology and external-replacement watcher using `notify` 8.2.0. |
| `ferrostage-settings` | TOML settings. |

## Commands

```text
just ci       # fmt --check, clippy -D warnings, test, cargo deny
just build    # cargo build --workspace --release
just dev      # cargo run -p ferrostage-app
just install  # cargo install --path crates/ferrostage-app --locked
```

Toolchain: Rust 1.99.0, edition 2024 (`rust-toolchain.toml`).

Those commands use default features. Build the full desktop GUI with `cargo build -p ferrostage-app --features gpui,pdfium,media --locked`, then launch `target/debug/ferrostage --config <settings.toml> --pdfium-library <pdfium.dll> <document.pdf>`. PDFium is supplied separately; native audio/video also needs the separately built media helper and selected runtime beside the GUI. Follow [packaging and runtime preparation](docs/packaging.md) for exact sources, notices and commands. The application never downloads runtimes or writes the user's PDF.

## License

MIT OR Apache-2.0. See [LICENSE](LICENSE).

Pympress is GPL-2.0-or-later. FerroStage does not copy or translate that source. `cargo deny` rejects GPL and AGPL, which also rules out MuPDF.

## Docs

- [docs/PROJECT_PLAN.md](docs/PROJECT_PLAN.md) — architecture and milestones
- [docs/architecture.md](docs/architecture.md)
- [AGENTS.md](AGENTS.md) — agent and contributor rules
