# Design: presenter notes-first layout

## Visual thesis

Notes are the presenter's primary surface; slides are orientation. Keep the existing warm-neutral token layer (DESIGN.md, `theme.rs`). The signature is size: the notes column takes most of the screen. No new colours, shadows or motion.

## Glyph size model

FerroStage scales each notes page to its pane. Rendered glyph height is:

`glyph_px = font_pt / page_width_pt x notes_page_width_px`

Units below are pixels of the user's 2000x1253 screenshot. The workbench is about 1990x1110. With `r = 0.60`, an 8 px divider and an 8 px panel padding plus a 24 px label row, the Notes pane inner size is about 1173x1070. The DPI scale of the screenshot is not known; the ratios do not depend on it.

| Case                                                  | Notes page width        | Glyph            | vs now |
| ----------------------------------------------------- | ----------------------- | ---------------- | ------ |
| Now: right column, 16:9 page, 16pt                    | about 650 px            | 23 px (measured) | 1.00x  |
| This task only: 16:9 page, 16pt, width-limited        | 1173 px (height 660 px) | 41 px            | 1.80x  |
| Both tasks: 453.5x425.2 pt page, 18pt, height-limited | 1141 px (1070 x 1.067)  | 45 px            | 1.97x  |

Most of the gain comes from the notes column. The thesis page adds about 9% glyph height, keeps the longest note (284 characters) inside the page at 18pt, and fills the 410 px band that a 16:9 page leaves empty. The column ratio is the only user knob; it scales glyphs linearly.

The best page height for this pane is 453.5 x 1070 / 1173 = 414 bp (14.6 cm). The thesis task uses 15 cm (425.2 bp): height-limited, 32 px (2.7%) of width unused, more line capacity. Keep 15 cm.

## Composition (width >= 800 logical px)

```
workbench-splits (row)
├─ slides-column   width = 1 - r            (flex col, gap 4)
│   ├─ current-pane   w_full, aspect = current frame aspect, flex_shrink_0
│   └─ next-pane      flex_1, min_h_0: label row + picture (contain)
├─ column-split    8 px hit target, 1 px line centred
└─ notes-panel     width = r, h_full, overflow_y_scroll, one p.card layer, p(8)
    └─ label + notes rasters (fit rule below)
```

The media slot rows and the debug line stay below `workbench-splits` inside `workbench` (`workbench_view.rs:1012-1051`). They take height from all three panes only when the document has media slots or the debug overlay is on. No change.

`r = settings.effective_notes_column_ratio()`, clamp 0.40 to 0.75.

If the left column is so narrow and tall that `current` height would exceed 65% of the column, cap `current-pane` with `max_h(relative(0.65))`; the picture then contains inside it. This keeps Next visible in tall windows.

Stacked (< 800 px): unchanged order Current, Next, Notes in one column; Notes keeps its scroll.

## Notes fit rule

For each notes raster with aspect `a = w/h` in a pane with inner size `W x H`:

`page_width = min(W, H x a)`, `page_height = page_width / a`, left-aligned at the top.

GPUI has no CSS `min()`. Measure the notes panel inner bounds with a `canvas` probe (the pattern already used for `split_bounds`), store the size in a `Rc<Cell<Size>>` on the Presenter, and use the previous layout's size on the next render. Request a re-render only when the measured size changes by more than 1 px (avoid a layout loop). Before the first measurement, fall back to the existing full-width rule. Multiple pages use the same width and stack in the scroll.

Note: `Surface::Notes` raster requests already follow the painted bounds through `Intent::SurfaceSize`, so a smaller page width also requests a matching raster size.

## Letterbox colour

`picture()` (`workbench_view.rs:244`) keeps `.bg(rgb(0x000000))` only for `Surface::Thumbnail(_)` (page browser). For `Surface::Current`, `Surface::Next` and `Surface::Notes(_)`, use no background (the workbench canvas shows). Audience rendering lives in `audience.rs` and is not touched.

DESIGN.md line 112 says "Keep slide aspect ratio and letterboxing." Rewrite it as: keep slide aspect ratio; presenter letterbox areas show the workbench canvas; audience letterbox stays black.

## Settings

- `PresenterSettings.notes_column_ratio: f64`, default 0.60. `effective_notes_column_ratio()` clamps to 0.40..=0.75.
- `parse_presenter`: `parse_f64("notes_column_ratio", "presenter.notes_column_ratio", default)`.
- `merge_presenter`: insert `notes_column_ratio`; keep inserting the two legacy keys with their parsed values (unchanged round trip).
- Schema version stays 1 (additive optional key).
- `settings_panel.rs:110-129`: remove the `presenter.next_slide_ratio` and `presenter.notes_ratio` fields; add `presenter.notes_column_ratio` Number {0.40..0.75, step 0.01}. Labels go in `assets/locales/zh-CN/app.toml` and `assets/locales/en-US/app.toml`.
- `presenter/settings_view.rs:597-605`: the slider whitelist names the two legacy keys. Replace them with `presenter.notes_column_ratio` so the new row keeps a slider.
- `Intent::Split { next, notes, save }` becomes `Intent::Split { notes_column, save }`. The shell handler (`shell.rs:630-640`) clamps to 0.40..=0.75 and writes `notes_column_ratio`.
- `shell.rs:2003`: the initial presenter raster width uses `1 - next_slide_ratio`. Change it to `1 - effective_notes_column_ratio()` so the first Current raster matches the new left column.
- `Presenter.split_drag: Option<bool>` (`presenter.rs:168`) only distinguishes horizontal from vertical drag. Change it to `bool`.
- `effective_notes_ratio()` and the vertical drag code are removed with their only callers. The settings test `legacy_notes_ratio_and_strip_are_readable_without_restoring_small_notes` (`tests/parse.rs:561`) asserts `effective_notes_ratio()`; keep its read-compatibility assertions and drop the effective-ratio ones. Keep the legacy fields in the struct.

## Divider

Replace both 6 px border-filled bars with one column divider: `w(px(8))`, transparent, child `w(px(1)) h_full bg(p.border)`; on hover/drag the child uses `p.focus`. `Palette` has no hairline token; do not add one. Cursor `cursor_col_resize()`. Drag math: `r = 1 - (x - bounds.x) / bounds.width`, clamped. Keyboard: not added (the settings row is the keyboard path).

## Header timer buttons

`timer_panel` (`workbench_view.rs:404`) calls `Presenter::button`. `button` (`presenter.rs:404`) sets `quiet_header` for `Home | Open | Settings | Displays | PresenterMenu(false)` outside Idle (`presenter.rs:420-428`), which gives `更多` and `显示器设置` their face. Add `Intent::Core(PauseTimer | ResumeTimer | ResetTimer)` to that match. No new style and no change in `timer_panel`.

## Near-dead setting

`presenter.notes_font_scale` scales only the fallback notes raster target (`workbench.rs:284`). After the Notes pane reports its painted size, `surface_targets` replaces the fallback, so the setting has no visible effect. This task leaves it unchanged and reports it. Making it visible would be a zoom feature that the user rejected in the 1.1.0 task.

## Contracts to update first

- `DESIGN.md` Layout "08:" paragraph and `Responsive Behavior` row text.
- `.trellis/spec/frontend/component-guidelines.md` line 59 workbench ordering.
- `.trellis/spec/frontend/state-management.md` "Manual navigation and original Notes workbench" contracts and matrix.

## Compatibility and rollback

- Existing settings files: new key absent gives 0.60; legacy keys survive round trip. Downgrading to 1.1.0 ignores the unknown key (parser keeps unknown leaves).
- Rollback: revert the commit; no data migration.
- Thesis notes PDF with mixed page sizes: `DocumentOverrides::default()` sets `layout: Some(NotesLayout::None)` (`ferrostage-document/src/layout.rs:23-30`), so `builder.rs:128-131` takes the manual branch and adds no `MixedGeometry` warning. Explicit Automatic (`layout: None`) shows the existing warning, which is correct behaviour. Independent notes pages reach the workbench through `notes_pages_for(page)` as `Surface::Notes(index + 1)` (`workbench.rs:303`) with `NormalizedRect::FULL`, at their own page size.

## Risks

- Measured-size feedback could flicker on the first frame after resize. Mitigation: 1 px hysteresis; headless test asserts stable bounds after two layout passes.
- Narrow left column makes slide text small on the laptop. The audience display shows the slide; the divider and settings row allow the user to trade width back.
- In the light theme the dark notes paper shows as a dark block on a light card. The user presents in the dark theme. Not handled in this task.
