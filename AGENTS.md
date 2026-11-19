<!-- TRELLIS:START -->
# mini-trellis

This project uses mini-trellis as a memory layer. Working knowledge lives under `.trellis/`:

- `.trellis/spec/` — durable contracts (short markdown; read `index.md` first)
- `.trellis/research/` — topic inbox (`<topic>.md`; `README.md` is not a topic)
- `.trellis/workspace/` — per-developer journals

Record a session with `/mini-trellis:remember` (Claude/OpenCode), `/mini-trellis-remember` (Pi/OMP/Kiro), `$mini-trellis-remember` (Codex/ZCode), or the `mini-trellis-remember` skill (Cursor and DeepSeek Harness). When earlier decisions or relevant project rules are needed, use `mini-trellis mem recall "<topic or path>"`; expand evidence with `mini-trellis mem read "<ref>"`. No search is required for unrelated work. Promote lasting boundaries into spec with `mini-trellis-update-spec`.

When the user confirms or corrects a project rule, save it through remember even without an explicit “remember” request, unless they ask not to save. An agreed future policy is recorded as agreed, not implemented. Also record reusable findings or a useful session/compact handoff. Questions, reviews, ordinary edits and unconfirmed proposals need no new record. New installs write notes without an automatic git commit; use `mini-trellis doctor` to check host wiring and `mini-trellis refresh --dry-run` before refreshing assets.

Keep one current rule in the document that already owns it. Preserve conditions with their rules: “when X, do not Y” must not become “never Y.” Unconfirmed scope, exceptions, and implementation choices stay open or are labeled as inference, not added to the contract.

For a confirmed update, use remember to edit the existing topic, verify the saved section once, record the journal link, and check the new entry/index row. Complete these authorized steps in the same turn without asking again between writes. Reuse returned file contents while unchanged instead of repeating reads. Journal-only success is not topic success; report partial writes accurately. New notes follow the requested or discussion language; updates preserve the document language.

For a memory-only update, reply with one short sentence using actual topic and journal Markdown links; expand for conflicts or partial writes. In a development task, include the links briefly with the requested result. Update affected README/docs/spec/research descriptions, including stale implementation status, in the same turn as the code. This maintenance needs no separate remember request or new journal entry.

When recall excerpts already answer the question with source and qualifiers, cite them and stop. Expand only missing context; do not repeat a lookup or fall back to dialogue-only `search` merely to reconfirm sufficient evidence.

ZCode, Kiro and OMP conversation histories are not parsed; use project knowledge or supply Markdown/text exports. For an explicit request to organize selected past sessions or supplied Markdown/text exports into project knowledge, use `mini-trellis-session-insight`; update existing documents and preserve sources.

mini-trellis does not create or drive tasks.

Managed by mini-trellis. Edits outside this block are preserved.
<!-- TRELLIS:END -->

# FerroStage

GPU-accelerated PDF and Beamer presenter. Rust edition 2024, toolchain 1.99.0 (`rust-toolchain.toml`). MIT. Clean-room: do not copy or translate Pympress (GPL-2.0-or-later).

## Commands

- `just ci` — fmt, clippy `-D warnings`, test, `cargo deny`. Default features, no PDFium DLL.
- `just build` — build the release desktop app with `gpui,pdfium`, stage verified PDFium and complete notices, and assemble the distribution payload. Resolve Cargo's actual target directory.
- `just release` — produce versioned Windows x64 Setup.exe, portable ZIP and SHA256 checksums under the resolved target directory's `dist`. Does not publish a release.
- `just dev` — stage that DLL into `target/debug`, then `cargo run -p ferrostage-app --features gpui,pdfium --locked`
- `just install` — build/package and silently run Setup for the current user, without clicks; Desktop/Start shortcuts and Installed apps registration. No Cargo-bin installation.
- `just uninstall` — run the registered native uninstaller; preserve settings and user PDFs.

## Crate rules

- `ferrostage-core` is the state machine. It must not depend on GPUI, PDFium, `lopdf`, or `notify`.
- The audience window reads `AudienceProjectionSnapshot` only. That type has no speaker notes. Notes stay in `ferrostage-document`.
- The audience window exists only in `Presenting`. `AudienceProjectionSnapshot` includes `AudienceStage`.
- Displays are `MonitorId` + `MonitorRole`, not indexes.
- GPUI is optional. Task07 uses the user-approved maintained source based on Zed tag `v1.23.3` (exact revision and five original local package patches in ADR-008), plus the separately approved four-file AccessKit Windows 0.34.0 F14 repair as the sixth maintained package. The separately user-approved Linux D-Bus downstream migration is bounded by ADR-011; it preserves that GPUI source and Windows repair. Only the three exact license exceptions in ADR-005 apply. Do not use crates.io `gpui` 0.2.x, mix GPUI families, or add advisory/lint waivers. Product checks and native acceptance are separate.
- PDF raster, when linked: `pdfium-render` 0.9.4, BGRA8, one render thread. `lopdf` 0.45 is already linked for structure checks.
- Hot reload is generic external PDF replacement: 300 ms debounce, wait for `%%EOF`, strictly newer `DocumentGeneration`. FerroStage opens user PDFs read-only and never writes them.
- Atomic temporary-file replacement is only for settings files and FerroStage's own artifacts, never the user's PDF. Settings use a unique sibling `<config-file-name>.tmp-<pid>-<seq>` (for example, `settings.toml.tmp-<pid>-<seq>`) and replace the configured settings path.
- No child HWND for video. Shared frames are BGRA8.
- `cargo deny` rejects GPL and AGPL (no MuPDF).

Design notes: `docs/PROJECT_PLAN.md`, `docs/architecture.md`, `docs/adr/`.

## Agent skills

### Issue tracker

Issues and specs live as GitHub issues in `bahayonghang/ferrostage`. See `docs/agents/issue-tracker.md`.

### Triage labels

Five default roles: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: one root `GLOSSARY.md` and ADRs in `docs/adr/`. See `docs/agents/domain.md`.




[You have received this identical output 3 times. Re-reading 'D:/Documents/Code/Rust/Exp/ferrostage/AGENTS.md:raw' will not change it — use a narrower selector (path:A-B), or proceed with the edit.]

