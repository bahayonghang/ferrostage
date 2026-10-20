# Design: notes-left dual-screen mode and toolbar/badge polish

## Boundary

- `ferrostage-core`: no change. Start, Stop and navigation commands are unchanged.
- `ferrostage-document`, audience window, `AudienceProjectionSnapshot`: no change. The new layout only rearranges presenter-side panes that already exist (Current, Next, Notes).
- `ferrostage-settings`: one `StartMode` value, two `PresenterSettings` fields with parse/merge/clamp.
- `ferrostage-ui`: policy match arm, start-mode menu, workbench composition, split intent, raster seed, theme/button faces, stage badge, captions, locale.
- No GPUI patch, no new dependency.

## Mode model (Q1)

```rust
pub enum StartMode {
    #[default]
    Dual,        // "dual"       Standard workbench
    DualNotes,   // "dual_notes" Notes-first workbench
    Single,      // "single"
}

impl StartMode {
    /// Presenter workbench composition shown in Ready/Presenting.
    pub const fn workbench_layout(self) -> WorkbenchLayout {
        match self {
            Self::DualNotes => WorkbenchLayout::NotesFirst,
            Self::Dual | Self::Single => WorkbenchLayout::Standard,
        }
    }
}
```

`WorkbenchLayout` lives in `ferrostage-ui` (`workbench.rs`), not in settings, because it is a view decision derived from the mode. The UI derives it from `PresenterData.start_mode`; nothing else stores it.

Alternatives rejected: a separate `presenter.layout` key would allow Single + NotesFirst, a combination with no visible effect, and needs a second control. A boolean on `Dual` cannot be expressed in the one radio menu.

Rollback: 1.2.0 reads `dual_notes` through `parse_choice`, warns `display.start_mode` and uses `dual`. Saving there rewrites `dual`. The two new presenter keys are unknown leaves for 1.2.0 and survive its round trip (unknown leaves are preserved).

## Placement

`policy::start_target`: `Single` keeps its branch; `Dual | DualNotes` delegate to `audience_target`. Every other caller already uses `audience_target`. `start_reason` follows `start_target`, so its messages are identical for both dual modes. Add `DualNotes` to the policy test matrix next to `Dual`.

## Settings

```rust
pub struct PresenterSettings {
    // existing fields unchanged
    /// Notes-first: fraction of the workbench width used by the Notes column.
    pub notes_first_notes_ratio: f64,   // default 0.55, effective 0.40..=0.70
    /// Notes-first: fraction of the right column height used by Current.
    pub notes_first_current_ratio: f64, // default 0.62, effective 0.50..=0.80
}
impl PresenterSettings {
    pub fn effective_notes_first_notes_ratio(&self) -> f64;
    pub fn effective_notes_first_current_ratio(&self) -> f64;
}
```

- `parse_presenter`: two `parse_f64` calls with the existing key/path pattern.
- `merge_presenter`: insert both keys beside `next_slide_ratio` / `notes_ratio`.
- Schema version stays 1 (additive optional keys).
- Settings panel: no new rows in this task (dividers are the editing path; the Standard rows stay as they are). Keep `settings_view.rs:680` slider whitelist unchanged.

## Workbench composition

`workbench_panel` branches once on `(stacked, layout)`:

| Case                       | Composition                                                   |
| -------------------------- | ------------------------------------------------------------- |
| stacked (< 800 px), any    | existing column: Current, Next, Notes (unchanged)             |
| wide, `Standard`           | existing code path, unchanged ids and geometry                |
| wide, `NotesFirst`         | tree below                                                    |

```
workbench-splits (row, gap 0)
├─ notes-panel        w = r_notes, h_full, overflow_y_scroll, one p.card layer, p(8)
│   └─ caption "当前讲稿" + notes rasters (existing width-fill rule, ordered, no shrink)
├─ column-split       8 px hit target, child 1 px line, cursor col-resize
└─ previews-column    flex_1, min_w_0, flex col
    ├─ current-pane   flex_grow r_current, basis 0, min_h_0: caption "当前页" + picture(Current, interactive)
    ├─ row-split      8 px hit target, child 1 px line, cursor row-resize
    └─ next-pane      flex_grow 1 - r_current, basis 0, min_h_0: caption "下一页" + picture(Next)
```

- Keep the ids `notes-panel`, `current-pane`, `next-pane` so existing probes, privacy tests and `Intent::SurfaceSize` reporting keep working. New ids: `previews-column`, `column-split`, `row-split`. Standard keeps `horizontal-split` / `vertical-split`.
- Pictures use the existing `picture()` (centred contain via `slide_surface::fit`, canvas letterbox). No change to `fit`, which the audience shares.
- The interactive Current surface (laser, pen, media hit-testing) moves to the right column without other change: pointer mapping already reads painted bounds.
- Geometry with defaults, in units of the 2000x1193 screenshot (workbench about 1990x990): Notes 1090 px wide; right column 890 px; Current picture 890x500; Next picture about 620x350. Current is about 1.4x Next in height. 16pt notes on a 453.5 pt-wide page render at about 38 px (vs about 31 px in the Standard right column).

## Split intent

Replace `Intent::Split { next, notes, save }` with

```rust
Intent::Split { layout: WorkbenchLayout, primary: f64, secondary: f64, save: bool }
```

- `Standard`: `primary = next_slide_ratio` (clamp 0.1..0.8 as today), `secondary = notes_ratio` (clamp 0.55..0.8).
- `NotesFirst`: `primary = notes_first_notes_ratio` (clamp 0.40..0.70), `secondary = notes_first_current_ratio` (clamp 0.50..0.80).
- Writes only the two fields of the given layout. Guarded by `!busy && Writable` as today.
- `Presenter.split_drag: Option<bool>` becomes `Option<SplitAxis>` (`Column | Row`), shared by both layouts.
- Drag math, NotesFirst: column `r = (x - bounds.x) / bounds.width`; row `r = (y - col.y) / col.height` measured on `previews-column` with a canvas probe, minus half the 8 px divider. `finish_split` dispatches the current values with `save: true` once.

## Raster seed

`shell.rs:2062`: initial Current target width uses `1 - next_slide_ratio`. Make it layout-aware: `Standard` -> `1 - next_slide_ratio`; `NotesFirst` -> `1 - effective_notes_first_notes_ratio()`. `SurfaceSize` replaces the seed after the first paint, so this only avoids a blurry first frame.

## Start-mode menu

- `CHOICES: [_; 3]` in display order Dual, DualNotes, Single; `start_row_focus: [FocusHandle; 3]`; `label()` falls back to `start-mode-dual`.
- Width `360f32.min(viewport - 24).max(200)`; rows lose the inner outline (see Focus reservation).
- Locale (Q3), helpers kept short enough for two lines at 360 px:
  - `start-mode-dual` 双屏 · 标准 / Dual · Standard — 左侧当前页，右侧下一页和讲稿；观众屏放映幻灯片。
  - `start-mode-dual-notes` 双屏 · 讲稿优先 / Dual · Notes first — 左侧讲稿，右侧当前页和下一页；观众屏放映幻灯片。
  - `start-mode-single` 单屏 / Single screen — 本屏全屏放映幻灯片，不显示讲稿和控件。
  - en-US helpers follow the same structure.
- New caption key `wb-current-slide` 当前页 / Current slide.

## Focus reservation (D-a)

`activation_target` sets `.border_color(rgb(palette.canvas))` at rest. Change it to a transparent colour (`gpui::transparent_black()`); `.focus(...)` keeps `palette.focus`. Geometry (`FOCUS_WIDTH`, `FOCUS_GAP` padding, radius) is unchanged, so no layout moves. On the canvas the result is pixel-identical; on card/soft surfaces (menus, segmented groups, page browser) the dark ring disappears. DESIGN.md "surface-coloured gap around primary buttons" still holds: the gap is the padding, which shows the surrounding surface.

## Toolbar faces (D-c, D-d)

- New `Presenter::chevron(open: bool, ink: u32)` drawn with `PathBuilder` like `session_icon`, 12 px, `flex_shrink_0`. The `button` helper gains an optional trailing icon; the trigger labels drop the `⌃`/`⌄` suffix. Accessible labels stay the same strings (no glyphs inside).
- Kind mapping in `Presenter::button`: `Intent::StartModeMenu | Intent::Document | Intent::ToggleAutoplay` -> `ButtonKind::Navigation`. No new `ButtonKind`, so the existing contrast test covers it.
- `ButtonKind::Destructive` radius 4 -> `RADIUS` (8). Error text and error border stay.
- Mode trigger in Ready: `min_h(SESSION_CONTROL)` like Start; Presenting does not render it.

## Badges (D-e, D-f)

- `stage-label`: `h(24)`, `px(8)`, `gap(6)`, `rounded(4)`, `bg(p.soft)`, text `p.body` 12 px medium, leading `div().size(8).rounded_full().bg(dot)` with debug selector `stage-dot`. Dot: Ready `p.success`, Presenting `p.focus`, Idle `p.muted`. Add the pair (body on soft) to the contrast test list if absent (it is covered by `used_text_and_focus_pairs_meet_design_contrast`).
- One `Presenter::pane_caption(key)` helper: 12 px, `FontWeight::MEDIUM`, `p.muted`, `pb(4)`, `flex_shrink_0`. Used by Notes (both layouts), Next (both layouts) and Current (Notes first only). Standard keeps no Current caption to avoid changing its approved geometry.

## Contracts to update first (R10)

- `DESIGN.md` Layout "08:" paragraph: add the Notes-first composition and its ratios; Components `button-secondary / tool-button`: Navigation face for bottom-strip triggers, drawn chevron, 8 px Clear radius, transparent focus reservation; `status / timer`: stage badge dot.
- `.trellis/spec/frontend/component-guidelines.md:59`: both compositions and ids.
- `.trellis/spec/frontend/state-management.md:35` and `:149`: `StartMode::DualNotes`, the two new keys, independence from the Standard ratios, Split intent shape.

## Tests

- Settings: extend `start_mode_defaults_to_dual_and_round_trips` with `dual_notes`; new test for the two keys (default, clamp, round trip, Standard keys untouched).
- Policy: `DualNotes` equals `Dual` across the matrix.
- UI (`presenter_layout_tests.rs`, `test-support`): AC3 menu real input, AC4 geometry, AC5 drags, AC6 Single/stacked, AC7 transparent border + chevron + width + radius, AC8 stage dot.
- Existing Standard layout, notes-privacy and start-mode tests must pass unchanged except the 2 -> 3 row count.

## Risks

- Proportions are subjective; the previous notes-first attempt was reverted. Mitigation: opt-in mode, Standard untouched, both dividers adjustable, native judgement by the user (AC10).
- Transparent focus border is global. Any surface that relied on the canvas ring as a visible outline would lose it. Review: home recent rows (canvas: identical), settings category nav (canvas fill: identical), page browser tiles (card: ring removed, tiles keep their own border). Check screenshots in AC10.
- Navigation face on triggers removes their outline; contrast stays valid because the Navigation kind has no boundary requirement in the existing test and the label is explicit.
