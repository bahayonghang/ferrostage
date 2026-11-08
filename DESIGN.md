---
version: alpha
name: 暖橙毛玻璃
description: "暖橙毛玻璃 is warm frosted glass for a presenter: an ivory field, translucent panels of the same hue, and one warm-orange accent for action, focus and selection."
colors:
  field-top: "#F7F2EB"
  field-bottom: "#EFE6DB"
  layer: "rgba(255,255,255,0.66)"
  layer-solid: "#FBF8F4"
  raised: "#FFFFFF"
  segment: "#FFFFFF"
  well: "#F3EEE7"
  control: "#EEEBE7"
  control-hover: "#E5E2DE"
  control-pressed: "#DCD9D5"
  hairline: "rgba(31,27,23,0.08)"
  border: "#8C847A"
  ink: "#1F1B17"
  body: "#3D3833"
  muted: "#5F5850"
  primary: "#D97757"
  primary-hover: "#E0866A"
  primary-active: "#C9694A"
  on-primary: "#1F1712"
  accent-text: "#A3492A"
  focus: "#A3492A"
  accent-wash: "#F7E6DC"
  success: "#2F6B45"
  warning: "#8A5A12"
  error: "#A63A32"
  error-wash: "#F8E3DF"
  dark-field-top: "#1D1B19"
  dark-field-bottom: "#26221E"
  dark-layer: "rgba(44,40,36,0.74)"
  dark-layer-solid: "#2A2622"
  dark-raised: "#38332E"
  dark-segment: "#524D48"
  dark-well: "#221F1C"
  dark-control: "#3B3734"
  dark-control-hover: "#44403D"
  dark-control-pressed: "#4C4945"
  dark-hairline: "rgba(255,255,255,0.08)"
  dark-border: "#8F877C"
  dark-ink: "#F5F1EB"
  dark-body: "#E2DBD2"
  dark-muted: "#C4BCB2"
  dark-accent-text: "#EBA98C"
  dark-focus: "#EBA98C"
  dark-accent-wash: "#3A2A22"
  dark-success: "#8DCEA0"
  dark-warning: "#E8B774"
  dark-error: "#FFA5A0"
  dark-error-wash: "#3B2523"
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
  field-alpha-native: 0.88
  dark-field-alpha-native: 0.82
  layer-alpha-native: 0.66
  dark-layer-alpha-native: 0.74
  solid-breakpoint: 960px
elevation:
  floating-shadow: "0 12px 32px rgba(31, 27, 23, 0.12)"
  dark-floating-shadow: "0 12px 32px rgba(0, 0, 0, 0.40)"
motion:
  press-offset: 1px
---
<!-- design-atlas: entry=style:airy-glass; source=first-party; license=MIT -->
<!-- customized: color system replaced by warm neutrals and one warm-orange accent; light and dark variants; material mapped to native GPUI window blur (no per-element backdrop-filter, no whole-window sheet); buttons bounded by fill, not outline -->

## Overview

暖橙毛玻璃 keeps the airy-glass material: a bright field, translucent layers of the same hue, large radii, and fill instead of outline. It replaces the color system with warm neutrals (ivory, sand, warm charcoal) and a single warm-orange accent.

Read status, then the number, then the action. There is no marketing hero.

One accent hue only. Orange marks the primary action, focus, selection, the running-timer dot and links. Success, warning and error colors appear only in status text and status marks, never as a button or panel fill.

Light is the default. Dark is a full variant with the same roles (`dark-*` tokens). The presenter's Light / Dark / System setting selects the variant; System follows the presenter window's observed appearance.

## Colors

Field: `{colors.field-top}` (`#F7F2EB`) to `{colors.field-bottom}` (`#EFE6DB`) at 160deg; dark `#1D1B19` to `#26221E`. The field covers the whole window, edge to edge. No blobs.

Layers, in depth order:

- **Panel** `{colors.layer}`: white at 0.66 (dark `#2C2824` at 0.74) over the native blurred field; `{colors.layer-solid}` (`#FBF8F4`, dark `#2A2622`) for every solid fallback. No outline.
- **Well** `{colors.well}` (`#F3EEE7`, dark `#221F1C`): inset area inside a panel, such as notes, inputs and grouped settings.
- **Raised** `{colors.raised}` (`#FFFFFF`, dark `#38332E`): menus, dialogs and tooltips.
- **Segment** `{colors.segment}` (`#FFFFFF`, dark `#524D48`): the selected option on a `control` track, always lighter than the track.
- **Control** `{colors.control}` / `-hover` / `-pressed`: ink at 0.06 / 0.10 / 0.14 (dark: white at 0.08 / 0.12 / 0.16), precomposited over `layer-solid` as opaque values.

Lines:

- `{colors.hairline}` is a decorative separator. It is never a control boundary.
- `{colors.border}` (`#8C847A`, dark `#8F877C`) is only for inputs and frames that need a visible edge; it is at least 3:1 on `well`, `layer-solid` and `raised`.

Text: `{colors.ink}` for titles and figures, `{colors.body}` for running text and control labels, `{colors.muted}` for 12px secondary text.

Accent:

- `{colors.primary}` (`#D97757`) fills the primary action in both variants. Its text is `{colors.on-primary}` (`#1F1712`). White text on this orange is about 3.1:1 and is not allowed.
- `{colors.accent-text}` (`#A3492A`, dark `#EBA98C`) for orange text and links; `{colors.focus}` is the same value for the focus ring and selection marks.
- `{colors.accent-wash}` (`#F7E6DC`, dark `#3A2A22`) fills a selected row or menu entry.

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

The field fills the window. Insets are margins between panels (home 24px, workbench 8px, narrow windows 4px), not a band around a sheet. Home is the field plus one centered panel card around its 760px reading column.

The workbench is a top bar panel, one panel per pane (Current, Next, Notes) and a bottom bar panel. Panes inset their picture 4px horizontally; captions and pictures carry the vertical inset, so saved split ratios still describe pane heights exactly. Notes are a well inside their panel. Statuses between panels (crop check, shell warnings, unsaved settings) are notice panels led by a status-colored dot. Settings, Displays and the crop check fill one full-height panel.

With 4px panel padding, an inner rounded area uses a 12px radius, because outer radius = inner radius + padding.

Icons are linear, about a 1.5px stroke, in `currentColor`. Do not add illustrated characters.

## Elevation & Depth

```
L0 field     full window, no inset, no outline; translucent only over native blur
 └ L1 panel  header, notes, previews, footer, home card; 16px radius, no outline
     └ well / control   inset areas and control fills of the same hue
 L2 raised   menus, dialogs, tooltips, page jump, crop check; hairline + one shadow
```

Depth comes from opacity and lightness steps of one hue, not from outlines or a second hue. Only L2 has a shadow: `{elevation.floating-shadow}` (dark `{elevation.dark-floating-shadow}`). Panels have no shadow. Do not stack shadows or add a glow.

Reference CSS for a web rendering of the same material:

```css
.field {
  background: linear-gradient(160deg, #F7F2EB, #EFE6DB);
}
.panel {
  background: rgba(255, 255, 255, 0.66);
  border-radius: 16px;
}
.panel.is-solid {
  background: #FBF8F4;
}
.floating {
  background: #FFFFFF;
  border: 1px solid rgba(31, 27, 23, 0.08);
  border-radius: 16px;
  box-shadow: 0 12px 32px rgba(31, 27, 23, 0.12);
}
```

The native presenter has no per-element `backdrop-filter`. Glass comes from the window-level system blur showing through the translucent field and panels.

## Shapes

| Token | Radius | Use |
| --- | --- | --- |
| `{rounded.image}` | 12px | Reserved for an image inside a panel with 4px padding; see Known Gaps |
| `{rounded.well}` | 12px | Wells |
| `{rounded.panel}` | 16px | Panels and the home card |
| `{rounded.floating}` | 16px | Menus and dialogs |
| `{rounded.pill}` | 999px | Buttons, segments and tags |

## Components

Buttons are pills bounded by their fill. No default outline.

| Kind | Rest | Hover / pressed | Text |
| --- | --- | --- | --- |
| Primary | `primary` | `primary-hover` / `primary-active` | `on-primary` |
| Secondary, navigation, tool | `control` | `control-hover` / `control-pressed` | `body` |
| Destructive | `control` | `control-hover` / `control-pressed` | `error` |
| Session Stop | `error-wash` | ink at 0.03 / 0.06 over `error-wash` | `error` |
| Quiet (header, timer, menu rows) | none | `control-hover` / `control-pressed` | `body` |
| Selected segment | `segment` | ink at 0.03 / 0.06 over `segment` | `ink` |
| Disabled | `control` | none | `muted` |

Grouped choices (black/white cover, drawing tools, crop layouts, setting choices, page-browser numbering, view and grid density) use a segmented control: one `control` track, the selected option lifted to `segment`. Each option's 2px focus ring and 2px gap form the 4px inset between track and segment. Do not wrap a group in a colored shell.

Selection never relies on color alone and never inserts a glyph that changes the label width. A choice that can be selected always reserves an 8px indicator slot; the selected option fills it with a `focus` dot (`on-primary` on an orange fill). The black/white swatch is its own slot and gains a 2px `focus` ring when selected. Menu rows keep one reserved check column with the same dot. Toggle buttons expose their state as accessible toggled state.

Keyboard focus is a 2px `{colors.focus}` ring outside a 2px gap.

## Accessibility

Meet WCAG 2.2 AA on the worst specified background. The native panel is measured as the panel layer over the field over a pure black and a pure white desktop; the table lists the lowest of those composites. Text is at least 4.5:1, the focus ring and selection marks at least 3:1. A labeled button is identified by its text, so it does not need a 3:1 outline.

These ratios are computed by `contrast_table.py`, kept in the research folder of the `11-07-presenter-warm-glass-restyle` Trellis task, from the `theme.rs` values and are checked by the unit tests in `crates/ferrostage-ui/src/theme.rs`. Regenerate them after any color change; do not edit them by hand.

| 前景 | 背景 | 浅色 | 深色 | 用途 |
| --- | --- | --- | --- | --- |
| `ink` | `layer_solid` | 16.16 (`#1F1B17` / `#FBF8F4`) | 13.34 (`#F5F1EB` / `#2A2622`) | 正文，实色面板 |
| `ink` | 毛玻璃合成 | 14.67 (`#1F1B17` / `#F0EDEA`) | 11.46 (`#F5F1EB` / `#35312D`) | 正文，毛玻璃面板最差合成 |
| `body` | `layer_solid` | 10.95 (`#3D3833` / `#FBF8F4`) | 10.94 (`#E2DBD2` / `#2A2622`) | 次级正文 |
| `muted` | 毛玻璃合成 | 6.01 (`#5F5850` / `#F0EDEA`) | 6.87 (`#C4BCB2` / `#35312D`) | 次要文字，毛玻璃面板最差合成 |
| `muted` | `control_pressed` | 4.98 (`#5F5850` / `#DCD9D5`) | 4.77 (`#C4BCB2` / `#4C4945`) | 按下的中性控件上的次要文字 |
| `on_accent` | `accent` | 5.65 (`#1F1712` / `#D97757`) | 5.65 (`#1F1712` / `#D97757`) | 主按钮文字 |
| `on_accent` | `accent_pressed` | 4.71 (`#1F1712` / `#C9694A`) | 4.71 (`#1F1712` / `#C9694A`) | 主按钮按下 |
| #FFFFFF | `accent` | 3.12 (`#FFFFFF` / `#D97757`) | 3.12 (`#FFFFFF` / `#D97757`) | 白字压橙色，禁止 |
| `accent_text` | `accent_wash` | 4.88 (`#A3492A` / `#F7E6DC`) | 6.90 (`#EBA98C` / `#3A2A22`) | 橙色文字压选中洗底 |
| `accent_text` | `control` | 4.99 (`#A3492A` / `#EEEBE7`) | 5.94 (`#EBA98C` / `#3B3734`) | 橙色文字压中性控件 |
| `focus` | 毛玻璃合成 | 5.08 (`#A3492A` / `#F0EDEA`) | 6.50 (`#EBA98C` / `#35312D`) | 焦点环与选中标记，毛玻璃面板最差合成 |
| `focus` | `control_pressed` | 4.21 (`#A3492A` / `#DCD9D5`) | 4.51 (`#EBA98C` / `#4C4945`) | 焦点环压最深中性填充 |
| `focus` | `segment` | 5.93 (`#A3492A` / `#FFFFFF`) | 4.21 (`#EBA98C` / `#524D48`) | 选中分段上的选中标记 |
| `ink` | `segment` | 17.11 (`#1F1B17` / `#FFFFFF`) | 7.42 (`#F5F1EB` / `#524D48`) | 选中分段文字 |
| `border` | `well` | 3.19 (`#8C847A` / `#F3EEE7`) | 4.63 (`#8F877C` / `#221F1C`) | 输入框边界 |
| `error` | `error_wash` | 5.21 (`#A63A32` / `#F8E3DF`) | 7.55 (`#FFA5A0` / `#3B2523`) | 停止/危险文字 |
| `warning` | `layer_solid` | 5.58 (`#8A5A12` / `#FBF8F4`) | 8.19 (`#E8B774` / `#2A2622`) | 警告文字 |

## Writing Tone

State the status, then the number, then the action. Keep labels short. No slogan and no marketing headline. Do not tighten Chinese text to match the Latin number tracking.

## Do's and Don'ts

### Do

- Put text only on a panel, a well, a control or a raised surface.
- Use `{colors.primary}` with `{colors.on-primary}` for the one primary action in a view.
- Use `{colors.focus}` for the 2px focus ring and selection marks.
- Separate layers by opacity and lightness of one warm hue.
- Use the solid fallback below 960px, for reduced transparency, and when the platform cannot blur.
- Set numbers in `tabular-nums`.

### Don't

- Don't put text directly on the field. Over native blur the field shows an arbitrary desktop.
- Don't indent a whole-window sheet or draw an outer window stroke.
- Don't outline every button. Use `border` only for inputs and frames that need a visible edge.
- Don't add a second accent hue. No green or purple in presenter chrome; status colors only for status text and marks.
- Don't put white text on `#D97757`.
- Don't let more than about an eighth of the desktop (a fifth in dark) show through the field. A lower field opacity turns the warm neutrals gray over colored wallpapers.
- Don't mark selection with a prefixed glyph that changes button width.
- Don't wrap a group of buttons in a colored shell.
- Don't use Inter, Plus Jakarta Sans, or DM Sans.
- Don't apply negative letter-spacing to Chinese running text.
- Don't add a second shadow, a glow, or a neon gradient.

## Responsive Behavior

| Condition | Field | Panel |
| --- | --- | --- |
| 960px and above, glass on, system allows blur | Gradient at 0.88 (dark 0.82) over the system blur | `layer` at 0.66 (dark 0.74) |
| Below 960px | Opaque gradient | `layer-solid` |
| Reduced transparency, unsupported backend or glass off | Opaque gradient | `layer-solid` |

Pills stay pills. The focus ring stays 2px. Keyboard focus does not move or scale.

## Known Gaps

- No chart palette.
- Contrast is measured on the specified surfaces and on the native composites over black and white desktops, not on photographs behind the window.
- If Figtree is unavailable, fall back to the system sans-serif and keep `"PingFang SC", "Noto Sans SC"` after the Latin family.
- Title weight 650 needs the Figtree variable font. If only a static file is available, use 600.
- The OS-owned title bar keeps its own system color.
- PDF previews are still painted with square corners. The paint path is the slide surface shared with the audience window, which this restyle does not change.

## FerroStage native presenter

The presenter owns this theme. Its home, document header, timer, tools, settings, menus and dialogs use the single Rust token layer in `crates/ferrostage-ui/src/theme.rs`. PDF previews and private note images retain the document's colors and aspect ratio. The audience window paints only its public snapshot, including its original black and white covers, and never reads this theme.

**Appearance.** General settings provide Light, Dark and System plus an independent glass preference. The initial appearance is light with glass enabled. An explicitly stored theme survives upgrade. Applying appearance settings updates the presenter in place through the existing settings transaction. Selecting a solid surface keeps the chosen color scheme. A window below 960 logical pixels, a system reduced-transparency preference or an unsupported backend uses the solid fallback while keeping the saved glass choice.

**Material.** Maintained GPUI has no per-element backdrop blur, only window-level `WindowBackgroundAppearance::Blurred`. On Windows and macOS with glass active, the field is drawn at 0.88 (dark 0.82) over the system blur, so an arbitrary desktop only tints its warm hue, and panels at the `layer` alpha above it. Otherwise the field is opaque and panels use `layer-solid`. Panels are not outlined; the focus indicator and control fill provide the interactive boundary. Font overrides and interface scaling apply to presenter controls.

**Composition.** Home keeps its 760 px reading column inside one centered panel card, one Open PDF action, Settings and Displays, and up to three recent documents. It scrolls on small windows. Ready and Presenting retain the filename, session controls and the centered presentation timer. Standard layout keeps Current on the left and Next and Notes on the right. Notes-first keeps Notes on the left at widths of 800 logical pixels and above. Saved splitter ratios, keyboard targets and note scrolling apply in both variants. Changes of appearance preserve the current page, timer, annotations, media and window roles.

**Controls.** Primary actions use the orange fill with dark text. Preserve hover, pressed, disabled, selected and focused states, the 2 px focus indicator and the minimum 44 px interactive target. Time digits keep a stable width. PDF and media are never assigned control fonts or tinted by the presenter.

The presenter uses an installed interface-font override first, then installed Figtree, then the platform UI family. It reads the available family names once per session. The clock and timer request tabular digits on that same face, so a missing optional font does not trigger a new native font search when the first PDF opens.
