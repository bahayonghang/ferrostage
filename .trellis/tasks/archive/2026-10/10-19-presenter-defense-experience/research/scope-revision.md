# Revised scope, 2026-10-19

## Explicit user decisions
Expand common Beamer animation support; optimize the original `slides/deck-notes` typography/layout; reject default reflowed text and Original/Text mode switching; avoid unnecessary features; update FerroStage to 1.1.0. This supersedes the initial Notes proposal and removes Q1. The user subsequently approved this complete task tree on 2026-10-19; implementation and validation are authorized.

## Thesis evidence and authority
Root: `D:/Documents/LYH/200-Learning/00博士毕业/毕业论文/thesis`.
Read root AGENTS.md, slides/AGENTS.md and `.trellis/spec/backend/defense-slides/01-06-core.md` before changes. Python commands use uv run; XeLaTeX steps are capped at five minutes. Repository guidance requires testing/sync and two builds for final page labels; outputs/previews are local.

`src/thesis_analysis/defense_notes_sync.py:34` owns NOTES_BLOCK. Current Notes settings are large text, 1.25 line spread and 0.65em paragraph skip. `slides/deck-notes/defense.tex:39` is generated. Editing only that driver will be lost on sync. Template definition must precede the appended `notes:N` label hook. Notes are independent pages, not a horizontally joined presenter PDF.

Source speech remains in frame-keyed note commands; notes.md is separately maintained and speech.md is an export. Sync replaces slide bodies/styles from deck and reattaches notes by frame id. Do not rewrite audience content just to change Notes.

The planning inventory records 95 Notes with 84-282 extracted characters, mean 202. Dense samples are physical 156, 66, 178, 184. Inspect source-rendered mathematics and full pages rather than estimating fit from character counts. Notes use a fixed-height Beamer page box, so overflow needs explicit verification.

## Existing work to preserve
At inspection, the thesis working tree already had edits in the defense-slides spec, justfile, pyproject.toml, slides/AGENTS.md, deck-notes/chapters/chapter1.tex and front.tex, both deck/copy ysu-defense-modern.sty, and defense_notes_sync.py; it also had unrelated material-document changes and new speech/export files. Recheck the live diff before implementation, treat these as user work, and only add scoped hunks. A full sync must not silently overwrite unrelated divergences. A later read in the same planning turn showed those scoped diffs had changed externally; recheck rather than relying on this historical dirty-file inventory.

The FerroStage tree contains only these new planning task directories at baseline. Its workspace package version is 1.0.0 at Cargo.toml:36; ten crates inherit it. Distribution derives the app version from cargo metadata (scripts/windows-distribution.ps1:67-72), then NSIS receives APP_VERSION (packaging/windows/installer.nsi). No installation has been requested.

## Compatibility scope and authority
See animation-compatibility.md for Beamer-generated commands and required mechanisms. Broader coverage reduces future work for standard PDFs; it cannot guarantee arbitrary future script/plugin compatibility. The separate animate package uses active content and remains explicitly outside this release. Minimal existing diagnostics should disclose known unsupported requirements before a presentation, rather than failing silently.

## Skill use and evidence status
UI skill governs native presenter hierarchy; Presentations/PDF guidance governs source-deck typography and render review. Local Beamer source and the thesis build workflow take precedence over generic PPTX authoring routes. Preserve the local LaTeX deliverable; do not create a replacement deck in Spaces/PowerPoint.

Planning baseline: source/spec/version inspection and task planning passed; product/thesis changes and their validation had not run at that point. Current execution evidence belongs in `validation.md` and the child research receipts.
