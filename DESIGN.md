---
version: alpha
name: 暖橙轻盈毛玻璃
description: "暖橙轻盈毛玻璃 is warm airy frosted glass for a presenter: an opaque ivory or warm-charcoal field with up to three cached radial ambient blobs, translucent paper of the same hue with a 1px edge, top inner highlight and clipped bottom shadow, translucent inset wells and control tracks, and one warm-orange accent for action, focus and selection."
colors:
  field-top: "#FBF6F0"
  field-bottom: "#F1E4D6"
  blob-start: "rgba(247,215,190,0.65)"
  blob-mid: "rgba(241,206,182,0.65)"
  blob-end: "rgba(246,224,200,0.65)"
  sheet: "rgba(255,253,251,0.66)"
  sheet-solid: "#FFFDF9"
  sheet-edge: "rgba(255,255,255,0.78)"
  sheet-highlight: "rgba(255,255,255,0.65)"
  sheet-shadow: "rgba(74,50,34,0.14)"
  overlay: "rgba(255,253,251,0.96)"
  overlay-solid: "#FFFFFF"
  overlay-edge: "rgba(31,27,23,0.10)"
  overlay-highlight: "rgba(255,255,255,0.72)"
  overlay-shadow: "rgba(74,50,34,0.22)"
  raised: "#FFFFFF"
  segment: "#FFFFFF"
  well: "rgba(31,27,23,0.045)"
  control: "rgba(31,27,23,0.075)"
  control-hover: "rgba(31,27,23,0.115)"
  control-pressed: "rgba(31,27,23,0.155)"
  hairline: "rgba(31,27,23,0.08)"
  border: "#7F776E"
  ink: "#1F1B17"
  body: "#3D3833"
  muted: "#524B44"
  primary: "#D97757"
  primary-hover: "#E0866A"
  primary-active: "#C9694A"
  on-primary: "#1F1712"
  accent-text: "#8A3B1E"
  focus: "#8A3B1E"
  accent-wash: "#F7E6DC"
  success: "#215534"
  warning: "#6B430A"
  error: "#872A22"
  error-wash: "#F8E3DF"
  titlebar-ink: "#1F1B17"
  titlebar-inactive-ink: "#524B44"
  titlebar-icon: "#1F1B17"
  titlebar-inactive-icon: "#524B44"
  titlebar-control-hover: "rgba(31,27,23,0.08)"
  titlebar-control-pressed: "rgba(31,27,23,0.14)"
  titlebar-close-hover: "#C42B1C"
  titlebar-close-pressed: "#A62317"
  titlebar-on-close: "#FFFDF9"
  dark-field-top: "#1C1917"
  dark-field-bottom: "#26201B"
  dark-blob-start: "rgba(59,39,28,0.70)"
  dark-blob-mid: "rgba(50,35,26,0.70)"
  dark-blob-end: "rgba(47,36,28,0.70)"
  dark-sheet: "rgba(43,38,34,0.64)"
  dark-sheet-solid: "#2A2521"
  dark-sheet-edge: "rgba(255,246,238,0.10)"
  dark-sheet-highlight: "rgba(255,246,238,0.08)"
  dark-sheet-shadow: "rgba(0,0,0,0.45)"
  dark-overlay: "rgba(49,44,39,0.96)"
  dark-overlay-solid: "#312C27"
  dark-overlay-edge: "rgba(255,246,238,0.12)"
  dark-overlay-highlight: "rgba(255,246,238,0.10)"
  dark-overlay-shadow: "rgba(0,0,0,0.60)"
  dark-raised: "#312C27"
  dark-segment: "#544E48"
  dark-well: "rgba(245,241,235,0.045)"
  dark-control: "rgba(245,241,235,0.08)"
  dark-control-hover: "rgba(245,241,235,0.12)"
  dark-control-pressed: "rgba(245,241,235,0.16)"
  dark-hairline: "rgba(255,255,255,0.08)"
  dark-border: "#958C81"
  dark-ink: "#F5F1EB"
  dark-body: "#E2DBD2"
  dark-muted: "#C8C0B6"
  dark-accent-text: "#F0B296"
  dark-focus: "#F0B296"
  dark-accent-wash: "#3A2A22"
  dark-success: "#8DCEA0"
  dark-warning: "#E8B774"
  dark-error: "#FFACA7"
  dark-error-wash: "#3B2523"
  dark-titlebar-ink: "#F5F1EB"
  dark-titlebar-inactive-ink: "#ACA49A"
  dark-titlebar-icon: "#F5F1EB"
  dark-titlebar-inactive-icon: "#ACA49A"
  dark-titlebar-control-hover: "rgba(255,255,255,0.09)"
  dark-titlebar-control-pressed: "rgba(255,255,255,0.15)"
  dark-titlebar-close-hover: "#C42B1C"
  dark-titlebar-close-pressed: "#A62317"
  dark-titlebar-on-close: "#FFFDF9"
typography:
  display-num:
    fontFamily: 'Figtree, "PingFang SC", "Noto Sans SC", sans-serif'
    fontSize: 32px
    fontWeight: 700
    letterSpacing: -0.02em
    fontVariantNumeric: tabular-nums
  title:
    fontFamily: 'Figtree, "PingFang SC", "Noto Sans SC", sans-serif'
    fontSize: 16px
    fontWeight: 650
    letterSpacing: 0
  body:
    fontFamily: 'Figtree, "PingFang SC", "Noto Sans SC", sans-serif'
    fontSize: 14px
    fontWeight: 400
    lineHeight: 1.7
    letterSpacing: 0
  meta:
    fontFamily: 'Figtree, "PingFang SC", "Noto Sans SC", sans-serif'
    fontSize: 12px
    fontWeight: 400
    letterSpacing: 0
rounded:
  image: 12px
  readout: 6px
  well: 12px
  panel: 16px
  floating: 16px
  pill: 999px
spacing:
  min: 8px
  sm: 12px
  md: 16px
  lg: 24px
  xl: 32px
  xxl: 48px
material:
  field-angle: 160deg
  blob-opacity: 0.65
  dark-blob-opacity: 0.70
  blob-fade: 70%
  blob-max: 3
  sheet-alpha: 0.66
  dark-sheet-alpha: 0.64
  overlay-alpha: 0.96
  dark-overlay-alpha: 0.96
elevation:
  sheet-highlight: "inset 0 1px 0 rgba(255,255,255,0.65)"
  sheet-shadow: "0 12px 40px -16px rgba(74,50,34,0.14)"
  overlay-highlight: "inset 0 1px 0 rgba(255,255,255,0.72)"
  overlay-shadow: "0 24px 64px -24px rgba(74,50,34,0.22)"
  dark-sheet-highlight: "inset 0 1px 0 rgba(255,246,238,0.08)"
  dark-sheet-shadow: "0 12px 40px -16px rgba(0,0,0,0.45)"
  dark-overlay-highlight: "inset 0 1px 0 rgba(255,246,238,0.10)"
  dark-overlay-shadow: "0 24px 64px -24px rgba(0,0,0,0.60)"
motion:
  press-offset: 1px
---
<!-- design-atlas: entry=style:airy-glass; source=first-party; license=MIT -->
<!-- customized: warm neutrals (ivory, sand, warm charcoal) and single warm-orange accent customized from style:airy-glass; light and dark variants; self-contained GPUI opaque-window material with cached SVG radialGradient blobs, 1px sheet edge, inset top highlight and clipped negative-spread ground_shadow (no WindowBackgroundAppearance::Blurred, no per-element backdrop-filter); buttons bounded by fill, not outline -->

## Overview

暖橙轻盈毛玻璃 adapts `style:airy-glass` to a warm presenter palette: an opaque ivory or warm-charcoal `field` with three soft warm ambient blobs, translucent glass `sheet` panels of the same hue bounded by a 1px crisp edge, top inner highlight and clipped bottom drop shadow, translucent inset `well` and `control` tracks that sink naturally inside each sheet, and a single warm-orange accent for action, focus and selection.

Read status, then the number, then the action. There is no marketing hero.

One accent hue only. Orange marks the primary action, focus, selection, the running-timer dot and links. Success, warning and error colors appear only in status text and status marks, never as a general button or panel fill; the integrated titlebar Close button uses dedicated danger-red fills on hover and press only.

Light is the default. Dark is a full variant with the same roles (`dark-*` tokens). The presenter's Light / Dark / System setting selects the variant; System follows the presenter window's observed appearance. Because the presenter window always uses `WindowBackgroundAppearance::Opaque` and paints its own self-contained field and blobs, light glass never darkens over dark desktop wallpapers and Windows 10 does not incur DWM blur drag stutter.

## Colors

**L0 Field**: `{colors.field-top}` (`#FBF6F0`) to `{colors.field-bottom}` (`#F1E4D6`) at 160deg; dark `#1C1917` to `#26201B`. When airy glass is active, three cached soft radial-gradient ambient blobs (`{colors.blob-start}`, `{colors.blob-mid}`, `{colors.blob-end}`, fading to alpha 0 at 70% radius) sit on the field at `(12%, 6%)` (`42rem × 32rem`), `(92%, 96%)` (`40rem × 32rem`), and `(82%, 10%)` (`30rem × 24rem`). The field fills the whole opaque window edge to edge. When glass is off or system reduced-transparency / high-contrast is enabled, the field omits all blobs and renders the solid 160deg gradient only.

**L1 Sheet (Glass Paper)**:

- Glass fill `{colors.sheet}` (`rgba(255,253,251,0.66)`, dark `rgba(43,38,34,0.64)`), bounded by 1px `{colors.sheet-edge}` (`rgba(255,255,255,0.78)`, dark `rgba(255,246,238,0.10)`), top 1px inner highlight `{elevation.sheet-highlight}` (`rgba(255,255,255,0.65)`, dark `rgba(255,246,238,0.08)`), and negative-spread clipped bottom drop shadow `{elevation.sheet-shadow}` (`0 12px 40px -16px rgba(74,50,34,0.14)`, dark `rgba(0,0,0,0.45)`).
- Solid fallback fill `{colors.sheet-solid}` (`#FFFDF9`, dark `#2A2521`) with `{colors.hairline}` 1px edge when glass is disabled or reduced-transparency / high-contrast is active.

**Wells & Controls (inside sheets/overlays)**:

- **Well** `{colors.well}` (`rgba(31,27,23,0.045)`, dark `rgba(245,241,235,0.045)`): inset area inside a panel (notes, inputs, preview frames, grouped settings). Always sinks beneath the sheet in Light (`lum(sheet) > lum(well)`) and steps cleanly above in Dark (`lum(sheet) < lum(well)`), eliminating Light-mode brightness inversion. Precomposited over `sheet-solid` in solid fallback mode.
- **Control** `{colors.control}` / `-hover` / `-pressed`: `rgba(31,27,23, 0.075 / 0.115 / 0.155)` (dark: `rgba(245,241,235, 0.08 / 0.12 / 0.16)`), precomposited over `sheet-solid` in solid mode. Unselected options on a `control` segmented track rest at `Tint::TRANSPARENT` so they do not double-darken over the translucent track.
- **Segment** `{colors.segment}` (`#FFFFFF`, dark `#544E48`): the selected option lifted off a `control` track, always brighter than the track.
- **Raised** `{colors.raised}` (`#FFFFFF`, dark `#312C27`): inline raised items and diagram tiles.

**L2 Overlay (Menus, dialogs, tooltips, page browser)**:

- Maintained GPUI has no per-element `backdrop_blur`, so overlays use high-opacity `{colors.overlay}` (`rgba(255,253,251,0.96)`, dark `rgba(49,44,39,0.96)`) or solid `{colors.overlay-solid}` (`#FFFFFF`, dark `#312C27`), 1px `{colors.overlay-edge}`, `{elevation.overlay-highlight}`, and clipped bottom `{elevation.overlay-shadow}` (`0 24px 64px -24px rgba(74,50,34,0.22)`, dark `rgba(0,0,0,0.60)`).

Lines:

- `{colors.hairline}` (`rgba(31,27,23,0.08)`, dark `rgba(255,255,255,0.08)`) is a decorative separator and solid-mode L1 sheet edge (without highlight or drop shadow). It is never a control boundary.
- `{colors.border}` (`#7F776E`, dark `#958C81`) is only for inputs and frames that need a visible edge; it is at least 3:1 on `well`, `sheet` and `overlay`.

Text: `{colors.ink}` (`#1F1B17`, dark `#F5F1EB`) for titles and figures, `{colors.body}` (`#3D3833`, dark `#E2DBD2`) for running text and control labels, `{colors.muted}` (`#524B44`, dark `#C8C0B6`) for 12px secondary text.

Accent:

- `{colors.primary}` (`#D97757`) fills the primary action in both variants. Its text is `{colors.on-primary}` (`#1F1712`). White text on this orange is about 3.12:1 and is not allowed.
- `{colors.accent-text}` (`#8A3B1E`, dark `#F0B296`) for orange text and links; `{colors.focus}` is the same value for the focus ring and selection marks.
- `{colors.accent-wash}` (`#F7E6DC`, dark `#3A2A22`) fills a selected row or menu entry.

Integrated titlebar (32px strip, 46px caption buttons):

- Sits transparently directly over the full-bleed warm `field` (`L0`) and its ambient blobs rather than drawing a separate opaque bar or system-colored strip.
- Active window title and icons use `{colors.titlebar-ink}` and `{colors.titlebar-icon}` (`#1F1B17`, dark `#F5F1EB`). Inactive (unfocused) windows step down to `{colors.titlebar-inactive-ink}` and `{colors.titlebar-inactive-icon}` (`#524B44`, dark `#ACA49A`) so focus changes remain clear while staying readable.
- Neutral window controls (minimize, maximize/restore) are transparent at rest and use translucent overlays `{colors.titlebar-control-hover}` (`rgba(31,27,23,0.08)`, dark `rgba(255,255,255,0.09)`) and `{colors.titlebar-control-pressed}` (`rgba(31,27,23,0.14)`, dark `rgba(255,255,255,0.15)`).
- The Close control is transparent at rest and switches to semantic danger reds `{colors.titlebar-close-hover}` (`#C42B1C`) and `{colors.titlebar-close-pressed}` (`#A62317`) with `{colors.titlebar-on-close}` (`#FFFDF9`) icon foreground on hover and press. These close danger reds are reserved exclusively for the titlebar Close button and never appear on general buttons or panel fills.

## Typography

Family: `Figtree, "PingFang SC", "Noto Sans SC", sans-serif`. Do not use Inter, Plus Jakarta Sans, or DM Sans.

| Role | Token | Size | Weight | Notes |
| --- | --- | --- | --- | --- |
| Large numbers | `{typography.display-num}` | 32px | 700 | `tabular-nums`. Latin digits may use -0.02em |
| Title | `{typography.title}` | 16px | 650 | No negative tracking |
| Body | `{typography.body}` | 14px | 400 | Chinese running text line-height 1.7 |
| Secondary | `{typography.meta}` | 12px | 400 | Color `{colors.muted}` |

The -0.02em tracking belongs only to large Latin numerals. Chinese running text does not use negative letter-spacing.

## Layout

Spacing is 8 / 12 / 16 / 24 / 32 / 48 (`{spacing.min}` through `{spacing.xxl}`). The minimum gap is 8px.

Order on the workbench is status, number, action. Do not add a marketing hero.

The field fills the window. Insets are margins between panels (home/workbench 8px, narrow workbench windows 4px; home guarantees at least 16px outer field margin at narrow widths and 32px at desktop viewports), not a band around a sheet. Home floats directly over the warm field and blobs without a whole-window card: a top header row (wordmark in ink on the field plus Settings/Displays and Open PDF pills, where Open PDF steps down to a secondary pill whenever a recent preview is selected so Open this document stays the sole warm-orange primary action), two independent L1 glass papers separated by a 32px field gap (`home-history` at 340px scaled width left of `home-preview` with a translucent `well` canvas on wide viewports, stacked vertically in a single column below 1000px scaled width), a lightweight `home-preparation` L1 glass summary bar below with a 24px field gap, and L1 `notice` panels for any home warnings or unsaved state.

Home shows the configured output target and start mode, never claiming that configuration is observed live output. Settings/Displays stay directly accessible. Preserve the first-three recent limit, accurate local minute timestamps, full-path disclosure, original import paths and user font preferences. Row activation selects a read-only first-content-page preview; only Open this document imports it. Initially select the first recent document. Selection has a reserved check mark and warm accent wash. Clear old pixels immediately when selection changes, and show localized loading, empty, unavailable and retry states.

Normal projection status has no extra row below the top bar. Output failures and black/white cover recovery retain the accessible status row and Displays/Restore actions. The navigation group orders Previous, progress/Jump, Next. Progress emphasizes the logical current slide with 26px semibold tabular accent-text digits; the total and optional overlay detail are secondary. Reserve current-digit width from the total's digit count, preserve the complete accessible logical/step/physical detail, and update immediately without animation. Primary orange fill remains reserved for primary actions.

The workbench is a top bar panel, one panel per pane (Current, Next, Notes) and a bottom bar panel. Panes inset their picture 4px horizontally inside an inner clipping/scrolling wrapper; captions and pictures carry the vertical inset, so saved split ratios still describe pane heights exactly and outer panel shadows are never clipped. Notes are a `well` inside their panel. Statuses between panels (crop check, shell warnings, unsaved settings) are notice panels led by a status-colored dot. Standalone Displays and the crop check fill a full-height modal panel (`modal_surface` with inner clipping wrapper).

With 4px panel padding, an inner rounded area uses a 12px radius, because outer radius = inner radius + padding.

Settings sits directly on the warm window field with a 16px outer margin at widths of 760 logical pixels and above (8px below 760), centering the header, split L1 glass papers, and footer together. At 760 logical pixels and above, the 176px navigation paper (`settings-nav-paper`, `self_start`, inner rounded clipping wrapper) sits 16px from the scrolling category body paper (`settings-body-paper`, at most 960px wide, or 1120px for Displays, with 20px×12px inner content padding), exposing the field and blobs between and around the papers; save feedback sits on a lightweight L1 paper strip aligned under the body paper. Each category has a square-cornered full-row target whose text box is centered on both axes; the selected row has an accent wash and an absolutely positioned short left bar, resting rows expose the navigation paper, hover and press use control fills, and the focus ring follows the rectangle. The title starts at the navigation paper edge and Close ends at the body paper edge. Below 760 logical pixels the layout stacks vertically with 12px gaps, wrapping category rows inside the full-width navigation paper above the scrolling body paper and full-width feedback strip.

Icons are linear, about a 1.5px stroke, in `currentColor`. Do not add illustrated characters.

## Elevation & Depth

```text
L0 field       opaque full-window 160deg gradient + 3 cached SVG radial blobs (when glass active)
 └ L1 sheet    panels (bars, panes, notes panel, notices, modal/home panels); 16px radius, 1px edge, top inset highlight, clipped ground_shadow
     └ well / control   translucent ink/white inset areas and control tracks of the same hue
 L2 overlay    menus, dialogs, tooltips, page browser, warning flags; 0.96 fill (or overlay-solid), 1px edge, top highlight, deeper ground_shadow
```

Depth comes from four sources: the warm field and radial blobs visible between panels, the 1px crisp edge, the 1px inset top highlight, and one soft bottom drop shadow with a negative spread. Never attach `overflow_hidden()` directly to an L1 `panel` or L2 `floating` outer `Div`, or its `ground_shadow` child hanging below the bottom edge will be clipped; clip/scroll content inside an inner child instead.

GPUI primitive mapping (`crates/ferrostage-ui/src/presenter/material.rs`):

- **Window appearance**: Always `WindowBackgroundAppearance::Opaque`; `WindowBackgroundAppearance::Blurred` and OS build gates are completely removed.
- **L0 Blobs**: 3 in-memory SVG `<radialGradient>` images (`Image::from_bytes(ImageFormat::Svg, ...)`) cached on `Material` by `p.is_dark`, never rebuilt per frame and never drawn with blur filters; positioned with zero-size percentage anchors and `ObjectFit::Fill`.
- **Top Highlight**: `BoxShadow { inset: true, offset: point(px(0.0), px(1.0)), blur_radius: px(0.0), spread_radius: px(0.0), color }`.
- **Negative-Spread Soft Shadow (`ground_shadow`)**: GPUI's `BoxShadow::blur_radius` is the Gaussian sigma (`css_blur / 2.0`), and an outer shadow directly on a translucent sheet darkens the sheet underneath. `ground_shadow` renders only the slice below `top(relative(1.0))` in an `.overflow_hidden()` strip with `reach = offset.y + 3.0 * blur_radius` and `drop = (1.0 - offset.y - spread_radius).max(0.0)`.

## Shapes

| Token | Radius | Use |
| --- | --- | --- |
| `{rounded.image}` | 12px | Reserved for an image inside a panel with 4px padding; see Known Gaps |
| `{rounded.readout}` | 6px | Wall clock and presentation timer |
| `{rounded.well}` | 12px | Wells |
| `{rounded.panel}` | 16px | Panels (`L1` sheets) |
| `{rounded.floating}` | 16px | Overlays (`L2` menus, dialogs, tooltips) |
| `{rounded.pill}` | 999px | Buttons, segments and tags |

## Components

General-purpose buttons are pills bounded by their fill. Settings categories use the rectangular navigation rows described above. No default outline.

| Kind | Rest | Hover / pressed | Text |
| --- | --- | --- | --- |
| Primary | `primary` | `primary-hover` / `primary-active` | `on-primary` |
| Secondary, navigation, tool | `control` | `control-hover` / `control-pressed` | `body` |
| Destructive | `control` | `control-hover` / `control-pressed` | `error` |
| Session Stop | `error-wash` | ink at 0.03 / 0.06 over `error-wash` | `error` |
| Quiet (header, timer, menu rows) | none | `control-hover` / `control-pressed` | `body` |
| Unselected segment on track | transparent (exposes `control` track) | `control-hover` / `control-pressed` | `body` |
| Selected segment | `segment` | ink at 0.03 / 0.06 over `segment` | `ink` |
| Disabled | `control` (or transparent on segmented track) | none | `muted` |
| Titlebar control (min/max) | none (over `field`) | `titlebar-control-hover` / `titlebar-control-pressed` | `titlebar-icon` (inactive: `titlebar-inactive-icon`) |
| Titlebar close | none (over `field`) | `titlebar-close-hover` / `titlebar-close-pressed` | `titlebar-icon` (inactive: `titlebar-inactive-icon`; hover/pressed: `titlebar-on-close`) |

The integrated presenter titlebar is 32px tall (`TITLEBAR_HEIGHT`), sits transparently on `field`, and places square-edged 46px-wide caption buttons (`TITLEBAR_BUTTON_WIDTH`) on the trailing edge (hidden on macOS, where a 78px leading inset reserves space for native traffic lights positioned at `(12px, 10px)`). Active and unfocused windows switch between `titlebar-ink`/`titlebar-icon` and `titlebar-inactive-ink`/`titlebar-inactive-icon`. Clicking Close routes through the presenter's safe close chain; danger red (`titlebar-close-hover` / `titlebar-close-pressed`) is strictly confined to the Close hover and pressed states.

Grouped choices (black/white cover, drawing tools, crop layouts, setting choices, page-browser numbering, view and grid density) use a segmented control: one `control` track, the selected option lifted to `segment`, and unselected options left `Tint::TRANSPARENT` at rest so they never double-darken the translucent track. Each option's 2px focus ring and 2px gap form the 4px inset between track and segment. Do not wrap a group in a colored shell.

Selection never relies on color alone and never inserts a glyph that changes the label width. Apart from settings category rows, a choice that can be selected reserves an 8px indicator slot; the selected option fills it with a `focus` dot (`on-primary` on an orange fill). The black/white swatch is its own slot and gains a 2px `focus` ring when selected. Menu rows keep one reserved check column with the same dot. Toggle buttons expose their state as accessible toggled state.

Keyboard focus is a 2px `{colors.focus}` ring outside a 2px gap.

## Accessibility

Meet WCAG 2.2 AA on the worst specified background. Because the presenter window is opaque and paints its own field and blobs, deterministic source-over compositing is evaluated across both field gradient stops (`field_top`, `field_bottom`), all three blob peaks (`blobs[0..3]`), translucent `sheet`, solid `sheet_solid`, translucent `well`/`control`/`control_hover`/`control_pressed`, L2 `overlay`/`overlay_solid`/`raised`, and integrated titlebar states. Text is at least 4.5:1, and focus rings, selection marks and input boundaries are at least 3:1.

| 前景 | 背景 | 浅色 | 深色 | 用途 |
| --- | --- | --- | --- | --- |
| `ink` | `sheet_solid` | 16.84 (`#1F1B17` / `#FFFDF9`) | 13.48 (`#F5F1EB` / `#2A2521`) | 标题与主文字，实色回退面板 |
| `ink` | `sheet` 最差合成（含色斑） | 15.22 (`#1F1B17` / `#FAF0E7`) | 13.15 (`#F5F1EB` / `#2F2620`) | 标题与主文字，毛玻璃面板 |
| `body` | `sheet_solid` | 11.41 (`#3D3833` / `#FFFDF9`) | 11.04 (`#E2DBD2` / `#2A2521`) | 正文与按钮文字，实色回退面板 |
| `body` | `sheet` 最差合成 | 10.31 (`#3D3833` / `#FAF0E7`) | 10.78 (`#E2DBD2` / `#2F2620`) | 正文与按钮文字，毛玻璃面板 |
| `muted` | `sheet` 最差合成 | 7.63 (`#524B44` / `#FAF0E7`) | 8.23 (`#C8C0B6` / `#2F2620`) | 次要文字，毛玻璃面板 |
| `muted` | `well` 最差合成 | 6.97 (`#524B44` / `#F0E6DE`) | 7.26 (`#C8C0B6` / `#382F29`) | 次要文字，半透凹槽 |
| `muted` | `control_pressed` 最差合成 | 5.58 (`#524B44` / `#D8CFC7`) | 5.11 (`#C8C0B6` / `#4F4640`) | 次要文字，按下的中性控件 |
| `accent_text` | `sheet` 最差合成 | 6.86 (`#8A3B1E` / `#FAF0E7`) | 8.11 (`#F0B296` / `#2F2620`) | 强调文字与链接 |
| `accent_text` | `control` 最差合成 | 5.92 (`#8A3B1E` / `#EAE0D7`) | 6.46 (`#F0B296` / `#3F3630`) | 橙色文字压中性控件 |
| `accent_text` | `accent_wash` | 6.35 (`#8A3B1E` / `#F7E6DC`) | 7.50 (`#F0B296` / `#3A2A22`) | 橙色文字压选中洗底 |
| `success` | `sheet` 最差合成 | 7.74 (`#215534` / `#FAF0E7`) | 8.08 (`#8DCEA0` / `#2F2620`) | 成功状态文字与标记 |
| `warning` | `sheet` 最差合成 | 7.68 (`#6B430A` / `#FAF0E7`) | 8.07 (`#E8B774` / `#2F2620`) | 警告状态文字与标记 |
| `warning` | `sheet_solid` | 8.50 (`#6B430A` / `#FFFDF9`) | 8.27 (`#E8B774` / `#2A2521`) | 警告文字，实色面板 |
| `error` | `sheet` 最差合成 | 7.84 (`#872A22` / `#FAF0E7`) | 8.25 (`#FFACA7` / `#2F2620`) | 错误状态文字与标记 |
| `error` | `error_wash` | 7.15 (`#872A22` / `#F8E3DF`) | 7.94 (`#FFACA7` / `#3B2523`) | 停止/危险文字 |
| `on_accent` | `accent` | 5.65 (`#1F1712` / `#D97757`) | 5.65 (`#1F1712` / `#D97757`) | 主按钮文字 |
| `on_accent` | `accent_pressed` | 4.71 (`#1F1712` / `#C9694A`) | 4.71 (`#1F1712` / `#C9694A`) | 主按钮按下 |
| `#FFFFFF` | `accent` | 3.12 (`#FFFFFF` / `#D97757`) | 3.12 (`#FFFFFF` / `#D97757`) | 白字压橙色，禁止 |
| `focus` | `sheet` 最差合成 | 6.86 (`#8A3B1E` / `#FAF0E7`) | 8.11 (`#F0B296` / `#2F2620`) | 焦点环与选中标记，玻璃纸 |
| `focus` | `control_pressed` 最差合成 | 5.02 (`#8A3B1E` / `#D8CFC7`) | 5.04 (`#F0B296` / `#4F4640`) | 焦点环压最深中性填充 |
| `focus` | `segment` | 7.71 (`#8A3B1E` / `#FFFFFF`) | 4.50 (`#F0B296` / `#544E48`) | 选中分段上的选中标记 |
| `ink` | `segment` | 17.11 (`#1F1B17` / `#FFFFFF`) | 7.29 (`#F5F1EB` / `#544E48`) | 选中分段文字 |
| `border` | `well` 最差合成 | 3.58 (`#7F776E` / `#F0E6DE`) | 3.95 (`#958C81` / `#382F29`) | 输入框与线框边界 |
| `ink` / `muted` | `overlay` 最差合成 | 16.60 / 8.32 | 12.28 / 7.68 | 浮层正文与次要文字 |
| `muted` / `accent_text` | `overlay_pressed` 最差合成 | 6.05 / 5.43 (`#DBD8D6`) | 4.74 / 4.67 (`#504C46`) | L2 浮层按下态次要与强调文字 |
| `ink` / `Utility` 按钮 | L0 底场与色斑最差合成 | 12.33 / 6.17 (`#F1D6C1`) | 13.03 / 6.59 (`#35251C`) | 首页顶栏与设置页标题/关闭行 |
| `titlebar_ink` / `icon` | 底场与色斑最差合成 | 12.33 (`#1F1B17` / `#F1D6C1`) | 13.03 (`#F5F1EB` / `#35251C`) | 一体化标题栏激活状态 |
| `titlebar_inactive_ink` / `icon` | 底场与色斑最差合成 | 6.18 (`#524B44` / `#F1D6C1`) | 5.95 (`#ACA49A` / `#35251C`) | 一体化标题栏失焦状态 |
| `titlebar_icon` | `titlebar_control_hover` 最差 | 10.58 (`#1F1B17` / `#E0C7B3`) | 9.84 (`#F5F1EB` / `#473930`) | 标题栏激活控制按钮悬停 |
| `titlebar_icon` | `titlebar_control_pressed` 最差 | 9.42 (`#1F1B17` / `#D4BCA9`) | 8.07 (`#F5F1EB` / `#53463E`) | 标题栏激活控制按钮按下 |
| `titlebar_inactive_icon` | `titlebar_control_hover` 最差 | 5.30 (`#524B44` / `#E0C7B3`) | 4.50 (`#ACA49A` / `#473930`) | 标题栏失焦控制按钮悬停 |
| `titlebar_on_close` | `titlebar_close_hover` | 5.57 (`#FFFDF9` / `#C42B1C`) | 5.57 (`#FFFDF9` / `#C42B1C`) | 标题栏关闭按钮悬停 |
| `titlebar_on_close` | `titlebar_close_pressed` | 7.19 (`#FFFDF9` / `#A62317`) | 7.19 (`#FFFDF9` / `#A62317`) | 标题栏关闭按钮按下 |
| `titlebar_ink` | `titlebar_inactive_ink` | 1.99 (`#1F1B17` / `#524B44`) | 2.19 (`#F5F1EB` / `#ACA49A`) | 标题栏激活与失焦区分（≥ 1.8） |

## Writing Tone

State the status, then the number, then the action. Keep labels short. No slogan and no marketing headline. Do not tighten Chinese text to match the Latin number tracking.

## Do's and Don'ts

### Do

- Put presenter text and controls on a sheet, a well, a control or an overlay, except for the three surfaces explicitly calibrated over the warm L0 window field: the 32px integrated titlebar (using `titlebar-*` tokens), the Home top header row (`ink` wordmark + `Utility`/`Primary` pills), and the Settings page title/Close header row (`ink` title + `Utility` Close pill).
- Use `{colors.primary}` with `{colors.on-primary}` for the one primary action in a view.
- Use `{colors.focus}` for the 2px focus ring and selection marks.
- Use `{colors.titlebar-close-hover}` and `{colors.titlebar-close-pressed}` with `{colors.titlebar-on-close}` exclusively for the integrated titlebar Close button's hover and press states.
- Separate layers by opacity, edge, highlight, bottom ground shadow and lightness steps of one warm hue (ensuring `lum(sheet) > lum(well) > lum(control)` in Light, and `lum(sheet) < lum(well) < lum(control) < lum(segment)` in Dark).
- Use the solid fallback (`sheet-solid` + 1px `hairline` without L1 highlight/shadow, `overlay-solid` + `overlay-edge` + elevation shadow, precomposited wells/controls, no blobs) only when `glass_effect` is off or OS reduced transparency / high contrast is enabled.
- Set numbers in `tabular-nums`.

### Don't

- Don't request `WindowBackgroundAppearance::Blurred` or depend on desktop wallpaper blur behind the presenter window.
- Don't put `overflow_hidden()` directly on an outer L1 `panel` or L2 `floating` element, or `ground_shadow` will be clipped.
- Don't paint `control` fill onto unselected options sitting on a `segmented` track; keep them `Tint::TRANSPARENT` at rest.
- Don't indent a whole-window sheet or draw an outer window stroke.
- Don't outline every button. Use `border` only for inputs and frames that need a visible edge.
- Don't add a second accent hue. No green or purple in presenter chrome; status colors stay in status text and marks, and titlebar close danger red (`#C42B1C` / `#A62317`) never appears on general buttons or panel fills.
- Don't put white text on `#D97757`.
- Don't mark selection with a prefixed glyph that changes button width.
- Don't wrap a group of buttons in a colored shell.
- Don't use Inter, Plus Jakarta Sans, or DM Sans.
- Don't apply negative letter-spacing to Chinese running text.
- Don't add a second outer shadow, a glow, or a neon gradient.

## Responsive Behavior

| Condition | Window Background | Field | Sheet & Overlay |
| --- | --- | --- | --- |
| Glass on, system allows transparency & contrast normal (all widths, all Windows 10/11 & macOS/Linux hosts) | `Opaque` | 160° warm gradient + 3 cached SVG `<radialGradient>` blobs | `sheet` (`0.66`/`0.64`), `overlay` (`0.96`), `well`/`control` translucent, `sheet_edge` + highlight + `ground_shadow` |
| Glass off (`general.glass_effect = false`) | `Opaque` | Solid 160° warm gradient (no blobs) | `sheet-solid` / `overlay-solid`, `hairline` edge, precomposited `well`/`control`, `glass_effect` preference preserved |
| OS reduced transparency or high contrast enabled | `Opaque` | Solid 160° warm gradient (no blobs) | `sheet-solid` / `overlay-solid`, `hairline` edge, precomposited `well`/`control`, restores airy glass automatically once OS conditions clear |

Pills stay pills. The focus ring stays 2px. Keyboard focus does not move or scale.

## Known Gaps

- No chart palette.
- If Figtree is unavailable, fall back to the system sans-serif and keep `"PingFang SC", "Noto Sans SC"` after the Latin family.
- Title weight 650 needs the Figtree variable font. If only a static file is available, use 600.
- PDF previews are still painted with square corners. The paint path is the slide surface shared with the audience window, which this restyle does not change.

## FerroStage native presenter

The presenter owns this theme. Its integrated titlebar, home, document header, timer, tools, settings, menus and dialogs use the single Rust token layer in `crates/ferrostage-ui/src/theme.rs` and material engine in `crates/ferrostage-ui/src/presenter/material.rs`. PDF previews and private note images retain the document's colors and aspect ratio. The audience window paints only its public snapshot, including its original black and white covers, and never reads this theme.

**Integrated Titlebar.** Home, Settings/Displays and the Ready/Presenting workbench share a 32px-tall transparent integrated titlebar painted directly over the full-bleed warm `field` (`L0`), always matching FerroStage's Light or Dark palette regardless of OS appearance. Trailing 46px caption buttons provide minimize, maximize/restore (preserving native drag, double-click zoom and Windows Snap Layouts) and safe close; macOS reserves a 78px leading inset for native traffic lights (`(12px, 10px)`) without duplicate controls, while Linux follows platform decoration negotiation. Unfocused windows lower title and icon emphasis from `titlebar-ink`/`titlebar-icon` to `titlebar-inactive-ink`/`titlebar-inactive-icon`, and fullscreen presenter mode hides the titlebar and restores it on exit. Titlebar Close danger red (`#C42B1C` / `#A62317`) is confined solely to the Close button's hover and pressed states.

**Appearance.** General settings provide Light, Dark and System plus an independent glass preference. The initial appearance is light with glass enabled. An explicitly stored theme survives upgrade. Applying appearance settings updates the presenter in place through the existing settings transaction. Full warm airy frosted glass is active across all window widths and platforms including Windows 10. Selecting a solid surface keeps the chosen color scheme. When OS reduced transparency (`UISettings::AdvancedEffectsEnabled == false` on Windows, `accessibilityDisplayShouldReduceTransparency` on macOS) or OS high contrast (`AccessibilitySettings::HighContrast` on Windows, `accessibilityDisplayShouldIncreaseContrast` on macOS) is detected (checked on a 2-second throttle), the presenter automatically switches to solid fallback (`sheet_solid` / `overlay_solid` without blobs) while preserving the saved `glass_effect` preference and reporting the active mode honestly under General → Frosted glass; restoring OS conditions automatically resumes airy glass.

**Material.** Maintained GPUI has no per-element backdrop blur. Rather than requesting `WindowBackgroundAppearance::Blurred`, the presenter keeps the window `Opaque` and builds airy frosted glass self-contained inside the window: a 160° gradient `field` with three cached in-memory SVG `<radialGradient>` blobs (`Arc<gpui::Image>` keyed by `is_dark`), `L1` translucent glass sheets (`1px` edge, `1px` inset top highlight, and `ground_shadow` negative-spread bottom drop shadow with content clipped inside an inner element), translucent ink/white `well` and `control` tracks that sink softly inside the sheet, and high-opacity `L2` overlays with deeper `ground_shadow`. Font overrides and interface scaling apply to presenter controls.

**Composition.** Home floats directly over the field without a single `home-card`: a transparent header row (wordmark, Settings/Displays, Open PDF), two separate L1 glass papers (`home-history` and `home-preview`) with a 32px field gap (horizontal on wide viewports, vertical single-column below 1000px scaled width), a compact L1 glass `home-preparation` footer bar, and `notice` panels for status messages. It scrolls on small windows. Ready and Presenting group the optional local HH:MM:SS clock (`well_bg`) and small dial on a 6px-radius inset before the filename. The centered presentation group stacks a short status above 30px tabular digits, aligned left on a shallow glass rectangle (`readout`: `raised` at 0.32 opacity fading to `sheet_solid` at 0.04 over the panel, opaque in solid fallback). Its numeric column reserves a signed three-digit minute reading in the chosen font. Fresh digits use ink, running digits use accent text, and the configured thresholds use warning and error. A ring identifies pause and a square identifies a stopped session. Pause, Resume and Reset sit beside the readout. Under 800px the timer group occupies its own centered row; controls wrap when space runs out. Standard layout keeps Current on the left and Next and Notes on the right. Notes-first keeps Notes on the left at widths of 800 logical pixels and above. Saved splitter ratios, keyboard targets and note scrolling apply in both variants. Changes of appearance or glass mode preserve the current page, timer, annotations, media and window roles.

**Controls.** Primary actions use the orange fill with dark text. Preserve hover, pressed, disabled, selected and focused states, the 2 px focus indicator and the minimum 44 px interactive target. Time digits keep a stable width. PDF and media are never assigned control fonts or tinted by the presenter.

The presenter uses an installed interface-font override first, then installed Figtree, then the platform UI family. It reads the available family names once per session. The General settings font button opens a focused, single-line search field. Typing narrows installed family names, arrows browse the scrollable results, and Enter or a click applies the highlighted or clicked family. The panel shows the current font separately. Escape returns to the button, Tab leaves the search, and Use default restores the system choice. The clock and timer request tabular digits on that same face, so a missing optional font does not trigger a new native font search when the first PDF opens.
