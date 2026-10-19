# Presenter notes-first layout

## Goal

The presenter stands less than 0.5 m from the laptop. The audience display is to the right. The presenter must read the authored speaker notes without effort. The current workbench gives the notes about 32% of the window width, so 16pt note text renders at about 23 px in a 2000 px wide screenshot. Large black letterbox bands and a grey band below the notes waste the rest of the screen.

The paired task `.trellis/tasks/10-19-defense-notes-readable-page` (stored in this repository, edits the thesis repository) changes the notes PDF pages (taller page, 18pt, dark paper). This task changes only FerroStage.

Expected glyph gain (design.md, screenshot units): this task alone takes 16pt notes from 23 px to about 41 px (1.80x). The thesis page change takes them to about 45 px (1.97x) and fills the column height.

## Evidence (2026-10-19)

- User screenshot, installed build (still shows the Page browser button that HEAD `235ae2b` removed, so the build predates HEAD). Window 2000x1253. Current pane about 1300x1100 holding a 16:9 slide, so about 370 px of black letterbox. Notes card about 650 px wide; the 16:9 notes raster uses about 365 px of height; the card continues as grey to the bottom.
- `workbench_view.rs` `workbench_panel`: Current left at `1 - next_slide_ratio`; Next above Notes in the right column; Notes wrapper `w_full().aspect_ratio(page)`.
- `workbench_view.rs` `picture`: `.bg(rgb(0x000000))` for every surface.
- User settings: `next_slide_ratio = 0.35`, `notes_ratio = 0.4` (effective 0.55). Fresh defaults are 0.40 and 0.65 (`schema.rs:262-263`).
- `notes_font_scale` is in the settings panel and schema. `workbench.rs:284` multiplies the fallback notes raster target by it. That fallback applies only until the Notes pane reports its size through `Intent::SurfaceSize`, so the setting has no visible effect after the first layout.
- `shell.rs:2003` sizes the initial presenter raster target from `1 - next_slide_ratio`.
- `notes-panel` and its child `note_panel` both paint `p.card` with radius; the child adds 12 px padding (`workbench_view.rs:789-800`, `987-1001`).
- The dark theme `card` token is `0x252320` (`theme.rs:183`), equal to the dark notes paper. In the dark theme the notes page edge is not visible against the card.
- Spike: XeLaTeX can give independent notes pages their own page size (`pdf:pagesize` in the note page template). Content pages stay 453.5x255.1 pt.

## Confirmed decisions (user, 2026-10-19)

- Layout: notes first. Left column about 40%: Current on top, Next below. Right column about 60%, full height: Notes. The column divider stays draggable.
- Notes page colour: dark paper, authored in the thesis repository. FerroStage shows the PDF pixels unchanged.
- Original PDF notes only. No text extraction, reflow, notes mode switch or zoom toolbox (decision kept from the 1.1.0 defense-experience task).

## Requirements

- R1 Notes-first composition at widths >= 800 logical px: left column holds Current (natural slide aspect, top-aligned, no letterbox at its natural size) and Next (labelled, fills the remaining left height, contained). The right column holds Notes at full workbench height.
- R2 Notes fit: each independent notes raster is as large as possible without exceeding the Notes pane height, at its own aspect ratio. One page needs no scroll. Several pages for one step keep source order in one vertical scroll.
- R3 One persisted ratio: `presenter.notes_column_ratio`, fraction of the workbench width for Notes. Fresh default 0.60. Effective range 0.40 to 0.75. Missing key in an existing settings file reads as the default. Legacy `next_slide_ratio` and `notes_ratio` stay read-compatible and are written back unchanged; they no longer affect layout.
- R4 Divider: one column divider. Visual weight is a 1 px hairline inside an 8 px hit target. Hover and drag show the border colour and a column-resize cursor. Drag updates live; release persists once.
- R5 No black on presenter-side panes. Current, Next and Notes letterbox areas use the workbench canvas. Audience output keeps the pure black curtain and is not changed.
- R6 Notes panel chrome: keep the "当前讲稿 / Notes" label; one card layer only (remove the nested second card); padding 8 px. No grey band remains below a single fitted page when the page aspect matches the pane.
- R7 Settings panel: replace the two legacy ratio rows with one "Notes column width" row bound to `notes_column_ratio`.
- R8 Header consistency: timer Pause/Resume and Reset use the same quiet header-utility face as the other header utilities. Keep explicit labels, 44 px targets, focus ring and press feedback.
- R9 Stacked layout (< 800 logical px) keeps Current, Next, Notes in one column, with Notes still scrollable.
- R10 DESIGN.md and `.trellis/spec/frontend` workbench contracts describe the new composition before code changes land.

## Acceptance criteria

- AC1 (R1, R2) Headless GPUI test at 1700x1000 logical: notes-panel width / workbench width within 0.60 +- 0.02; Current pane height = width x 9/16 +- 1 px; a single 453.5x425 pt notes frame is fully visible without scroll; a 16:9 notes frame is width-limited.
- AC2 (R2) Existing multi-page notes test passes with updated geometry: ordered pages, no equal-height compression, one scroll.
- AC3 (R3) Settings round-trip: missing key gives 0.60; 0.3 reads as effective 0.40; drag-release writes `notes_column_ratio` once and preserves `next_slide_ratio`, `notes_ratio` and unknown TOML leaves.
- AC4 (R4, R7) Real-input drag test moves the divider and persists; the settings row edits the same value.
- AC5 (R5) No `0x000000` background on Current, Next or Notes surfaces; audience and page-browser thumbnails unchanged.
- AC6 (R8) Timer buttons share the header-utility face; keyboard focus and Enter/Space still work.
- AC7 `just ci` passes (fmt, clippy -D warnings, test, deny).
- AC8 Native check with the real thesis `defense.pdf` in the user's dual-display setup: screenshot before and after, measured notes glyph height, PASS/FAIL/NOT_RUN recorded. Standing-distance readability is judged by the user, not claimed from numbers.

## Out of scope

Text extraction or reflow of notes, notes zoom controls, changes to `notes_font_scale` (first-layout fallback only; reported, see design.md), light-theme treatment of dark notes paper, audience rendering, transitions, page browser, release packaging, installation, push or publish.
