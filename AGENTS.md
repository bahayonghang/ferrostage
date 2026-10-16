# FerroStage

GPU-accelerated PDF and Beamer presenter. Rust edition 2024, toolchain 1.99.0 (`rust-toolchain.toml`). MIT OR Apache-2.0. Clean-room: do not copy or translate Pympress (GPL-2.0-or-later).

## Commands

- `just ci` — fmt, clippy `-D warnings`, test, `cargo deny`. Default features, no PDFium DLL.
- `just build` — release workspace without the app, then `ferrostage-app` with `gpui,pdfium`, then the pinned `pdfium.dll` beside `target/release/ferrostage.exe`. Double-click that exe.
- `just dev` — stage that DLL into `target/debug`, then `cargo run -p ferrostage-app --features gpui,pdfium --locked`
- `just install` — `cargo install --path crates/ferrostage-app --locked --features gpui,pdfium`, then only `pdfium.dll` beside the installed exe

## Crate rules

- `ferrostage-core` is the state machine. It must not depend on GPUI, PDFium, `lopdf`, or `notify`.
- The audience window reads `AudienceProjectionSnapshot` only. That type has no speaker notes. Notes stay in `ferrostage-document`.
- The audience window exists only in `Presenting`. `AudienceProjectionSnapshot` includes `AudienceStage`.
- Displays are `MonitorId` + `MonitorRole`, not indexes.
- GPUI is optional. Task07 uses the user-approved maintained source based on Zed tag `v1.23.3` (exact revision and five original local package patches in ADR-008), plus the separately approved four-file AccessKit Windows 0.34.0 F14 repair as the sixth maintained package. Only the three exact license exceptions in ADR-005 apply. Do not use crates.io `gpui` 0.2.x, mix GPUI families, or add advisory/lint waivers. Product checks and native acceptance are separate.
- PDF raster, when linked: `pdfium-render` 0.9.4, BGRA8, one render thread. `lopdf` 0.45 is already linked for structure checks.
- Hot reload is generic external PDF replacement: 300 ms debounce, wait for `%%EOF`, strictly newer `DocumentGeneration`. FerroStage opens user PDFs read-only and never writes them.
- Atomic temporary-file replacement is only for settings files and FerroStage's own artifacts, never the user's PDF. Settings use a unique sibling `<config-file-name>.tmp-<pid>-<seq>` (for example, `settings.toml.tmp-<pid>-<seq>`) and replace the configured settings path.
- No child HWND for video. Shared frames are BGRA8.
- `cargo deny` rejects GPL and AGPL (no MuPDF).

Design notes: `docs/PROJECT_PLAN.md`, `docs/architecture.md`, `docs/adr/`.

<!-- TRELLIS:START -->
# Trellis Instructions

These instructions are for AI assistants working in this project.

This project is managed by Trellis. The working knowledge you need lives under `.trellis/`:

- `.trellis/workflow.md` — development phases, when to create tasks, skill routing
- `.trellis/spec/` — package- and layer-scoped coding guidelines (read before writing code in a given layer)
- `.trellis/workspace/` — per-developer journals and session traces
- `.trellis/tasks/` — active and archived tasks (PRDs, research, jsonl context)

If a Trellis command is available on your platform (e.g. `/trellis:finish-work`, `/trellis:continue`), prefer it over manual steps. Not every platform exposes every command.

If you're using Codex or another agent-capable tool, additional project-scoped helpers may live in:
- `.agents/skills/` — reusable Trellis skills
- `.codex/agents/` — optional custom subagents

Managed by Trellis. Edits outside this block are preserved; edits inside may be overwritten by a future `trellis update`.

<!-- TRELLIS:END -->


[You have received this identical output 3 times. Re-reading 'D:/Documents/Code/Rust/Exp/ferrostage/AGENTS.md:raw' will not change it — use a narrower selector (path:A-B), or proceed with the edit.]
