# Notes-left dual-screen mode and toolbar/badge polish

## Goal

The Start dropdown ("演示方式") offers Dual (default) and Single today. Add a third choice, a second dual-screen mode in which the presenter workbench puts the original Notes on the left, the Current slide upper-right (larger) and the Next slide lower-right (smaller). Placement on the displays is the same as Dual. The existing Dual layout stays the default and is unchanged.

In the same task, fix the visual defects of the bottom toolbar, the dropdown menus and the header status badge seen in the user screenshot of 2026-10-20 (1.2.0, `defense.pdf`, Ready, Single selected, menu open).

## Evidence (2026-10-20)

- `StartMode { Dual, Single }` (`crates/ferrostage-settings/src/schema.rs:75-90`), TOML `display.start_mode`, unknown values warn and use `dual` (`parse/sections.rs:189-197`, test `start_mode_defaults_to_dual_and_round_trips`).
- Menu rows: `presenter/start_mode.rs:5-12` (`CHOICES: [_; 2]`); focus handles `start_row_focus: [FocusHandle; 2]` (`presenter.rs:154`). Placement: `policy::start_target` (`policy.rs:130`) only distinguishes `Single`.
- Workbench: `workbench_view.rs:789-1081` builds Current left at `1 - next_slide_ratio`, right column with Next (`flex_grow(1 - notes)`) above Notes (`flex_grow(notes)`), two 6 px `p.border` bars. `Intent::Split { next, notes, save }` (`shell.rs:659-667`) clamps and persists. Initial Current raster width uses `1 - next_slide_ratio` (`shell.rs:2062`).
- History: a notes-first layout (Notes as the right column, `notes_column_ratio` 0.60) shipped in `f33f097` and was reverted in `e6ed7e4` because the user found the proportions unpleasant. The revert set the right column to 45% (`next_slide_ratio = 0.45`). This task adds an opt-in mode and does not replace the default.
- Defects in the screenshot, with source:
  - D-a Menu rows show dark rings. Every `activation_target` reserves the focus ring with a 2 px border in `p.canvas` (`presenter.rs`, `activation_target`). On a `p.card` menu the canvas colour shows as an outline around each row. Same on the soft segmented groups, where it draws dark gaps between Black/White and between Laser/Pen/Highlighter/Eraser, so the groups read as separate boxes.
  - D-b The Single helper wraps a lone `。` onto its own line (menu width 320 px, `start_mode.rs:31`; text `start-mode-single-help`).
  - D-c Trigger chevrons are font glyphs `⌃` / `⌄` (`workbench_view.rs:541`, `:725-729`). They render small and above the baseline ("单屏 ^").
  - D-d Mixed secondary faces in one strip: Previous/Next use the borderless Navigation face, the mode trigger, Layout & navigation and automatic-playback use the outlined Utility face, Clear uses a 4 px radius (`theme.rs`, `ButtonKind::Destructive`) while every other control uses 8 px.
  - D-e The header stage badge (`presenter.rs`, `stage-label`) is muted text on `p.soft` with a 4 px radius. It reads as a grey block, not a status.
  - D-f Pane captions (`wb-next-slide`, `wb-notes`) are plain 12 px muted text placed inline in different containers; Current has no caption.

## Requirements

- R1 Mode choice. The dropdown lists, in order: Dual · Standard (default, current layout), Dual · Notes first (new), Single. Selecting a row closes the menu, saves, and switches the Ready workbench layout immediately. The trigger shows the selected mode. Selection stays disabled while busy and hidden while presenting (unchanged).
- R2 Placement. Notes first places presenter and audience exactly like Dual (display roles, single-monitor mode on one monitor). Single is unchanged.
- R3 Notes-first composition at widths >= 800 logical px: left column, full workbench height: Notes (label, original rasters, one vertical scroll, existing width-fill rule). Right column: Current on top (captioned, larger), Next below (captioned, smaller). Both previews keep the slide aspect and letterbox on the workbench canvas. Fresh defaults: Notes column 55% of the width; Current pane 62% of the right column height.
- R4 Dividers. One column divider (Notes | previews) and one row divider (Current / Next). Each is a 1 px line in an 8 px hit target; hover and drag show the focus colour and a resize cursor. Drag updates live; release persists once. Ranges: Notes column 0.40 to 0.70; Current share 0.50 to 0.80.
- R5 Persistence. Notes-first ratios are separate keys, so switching modes never changes the Standard ratios and the reverse. Missing keys read as defaults. Unknown or out-of-range values follow the existing `parse_f64` and clamp behaviour. Older builds that read `start_mode = "dual_notes"` warn and fall back to `dual` (no data loss).
- R6 Single keeps the Standard workbench in Ready. Stacked layout (< 800 px) uses the existing Current, Next, Notes order in every mode.
- R7 Privacy is unchanged: Notes stay presenter-only; the audience reads `AudienceProjectionSnapshot` only; no new data reaches the audience window.
- R8 Toolbar and menu polish (fixes D-a to D-d):
  - Focus reservation is transparent at rest, so rows and segmented items show their own container surface. The focus ring colour and geometry are unchanged.
  - The start-mode menu is 360 px preferred width (viewport-bounded), rows have no inner outline, helper text fits on at most two lines in both locales at the default text size.
  - Trigger chevrons (start mode, Layout & navigation) are drawn vector icons, vertically centred, up when the menu is open.
  - In the bottom strip, the mode trigger, Layout & navigation and automatic-playback use the Navigation face. Start keeps clay; Clear keeps error text and border with the shared 8 px radius.
  - The mode trigger matches Start's height in Ready so the pair reads as one group.
- R9 Badge polish (fixes D-e, D-f):
  - Stage badge: leading 8 px dot plus text in `p.body`, `p.soft` fill, 4 px (sm) radius, fixed 24 px height. Dot colours: Ready `success`, Presenting `focus`, Idle `muted`. Text always accompanies colour.
  - Pane captions (Current in Notes first, Next, Notes) share one caption style: 12 px medium `p.muted`, same left inset and 4 px gap above the picture.
- R10 Contracts first. `DESIGN.md`, `.trellis/spec/frontend/component-guidelines.md` and `.trellis/spec/frontend/state-management.md` describe the mode, the composition, the new keys and the polished faces before code lands. Both locale files get the new strings.

## Acceptance criteria

- [ ] AC1 (R1, R5) Settings: `dual_notes` round-trips; `dual` stays default; `triple` still warns and loads `dual`; the two new ratio keys default to 0.55 and 0.62, clamp to their ranges, round-trip, and preserve the Standard ratios and unknown TOML leaves.
- [ ] AC2 (R2) Policy: `DualNotes` resolves exactly like `Dual` for 0, 1 (both single-monitor modes) and 2 monitors.
- [ ] AC3 (R1) GPUI real-input test: the menu shows three radio rows in order with the saved mode checked; selecting Notes first saves `start_mode = "dual_notes"`, closes the menu, and the next render has `notes-panel` left of `current-pane`; keyboard Arrow/Enter reaches row 3.
- [ ] AC4 (R3) Headless layout at 1700x1000 logical, Notes first: `notes-panel` x < `current-pane` x; notes width / workbench width = 0.55 +- 0.02; `current-pane` above `next-pane`; Current picture height > Next picture height; Standard layout geometry tests still pass unchanged.
- [ ] AC5 (R4) Real-input drags on both Notes-first dividers update live and persist once on release, writing only the two Notes-first keys.
- [ ] AC6 (R6, R7) Single in Ready renders the Standard workbench; stacked width renders Current, Next, Notes in every mode; existing notes-privacy tests pass.
- [ ] AC7 (R8) Unit/GPUI checks: activation targets have a transparent rest border and the focus colour when focused; `start-mode-menu` width is 360 at a wide viewport; triggers contain the chevron icon element and no `⌃`/`⌄` text; Clear radius equals `theme::RADIUS`; contrast tests in `theme.rs` pass for any changed button kind.
- [ ] AC8 (R9) Stage badge renders a dot element with the stage's semantic colour for Ready and Presenting, keeps the localized text, and contrast of text on fill >= 4.5:1 in both themes.
- [ ] AC9 `just ci` passes (fmt, clippy `-D warnings`, test, deny).
- [ ] AC10 Native check (user, `just dev`, `defense.pdf`, dual-display setup, dark theme): screenshots of Standard and Notes first in Ready and Presenting, plus the open menu. Proportions and readability are judged by the user. Record PASS/FAIL/NOT_RUN.

## Out of scope

- Switching layout while presenting (the trigger stays hidden while presenting).
- Notes fit-to-height, text extraction, notes zoom, `notes_font_scale` behaviour.
- A notes-first stacked order, layout presets beyond the three modes, light-theme treatment of dark notes paper.
- Audience rendering, transitions, page browser content, release version bump, packaging, push or publish.

## Decisions (confirmed by the user on 2026-10-20: "确认，按建议的默认方案开始实现")

- Q1 Model: the new layout is a third `StartMode` value `DualNotes` (TOML `dual_notes`) rather than a separate `presenter.layout` setting. Reason: one user-facing choice, Single has no visible workbench anyway, smallest schema change.
- Q2 Default ratios: Notes column 0.55 (mirror of the approved 55:45 Standard split), Current 0.62 of the right column height.
- Q3 Labels: zh-CN `双屏 · 标准` / `双屏 · 讲稿优先` / `单屏`; en-US `Dual · Standard` / `Dual · Notes first` / `Single screen`.
- Q4 Version: no version bump in this task (1.2.0 stays).
