---
version: alpha
name: 暖橙毛玻璃
description: "暖橙毛玻璃 is warm frosted glass for a presenter: an ivory field, translucent panels of the same hue, and one warm-orange accent for action, focus and selection."
colors:
  field-top: "#F7F2EB"
  field-bottom: "#EFE6DB"
  layer: "rgba(255,250,244,0.21)"
  layer-top: "#FFFFFF"
  layer-solid-top: "#FFFDF9"
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
  muted: "#49433C"
  primary: "#D97757"
  primary-hover: "#E0866A"
  primary-active: "#C9694A"
  on-primary: "#1F1712"
  accent-text: "#76341C"
  focus: "#76341C"
  accent-wash: "#F7E6DC"
  success: "#215132"
  warning: "#66400A"
  error: "#802821"
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
  dark-field-top: "#1D1B19"
  dark-field-bottom: "#26221E"
  dark-layer: "rgba(118,105,93,0.07)"
  dark-layer-top: "#B7A899"
  dark-layer-solid-top: "#322D29"
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
  field-alpha-native: 0.72
  dark-field-alpha-native: 0.86
  layer-alpha-native: 0.21
  dark-layer-alpha-native: 0.07
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

One accent hue only. Orange marks the primary action, focus, selection, the running-timer dot and links. Success, warning and error colors appear only in status text and status marks, never as a general button or panel fill; the integrated titlebar Close button uses dedicated danger-red fills on hover and press only.

Light is the default. Dark is a full variant with the same roles (`dark-*` tokens). The presenter's Light / Dark / System setting selects the variant; System follows the presenter window's observed appearance.

## Colors

Field: `{colors.field-top}` (`#F7F2EB`) to `{colors.field-bottom}` (`#EFE6DB`) at 160deg; dark `#1D1B19` to `#26221E`. The field covers the whole window, edge to edge. No blobs.

Layers, in depth order:

- **Panel** uses a vertical highlight that settles into its base at 55% height. Light blends white into warm ivory at 0.21 opacity. Dark blends warm sand into walnut at 0.07. The solid equivalents use the two opaque panel stops.
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
- `{colors.accent-text}` (`#76341C`, dark `#EBA98C`) for orange text and links; `{colors.focus}` is the same value for the focus ring and selection marks.
- `{colors.accent-wash}` (`#F7E6DC`, dark `#3A2A22`) fills a selected row or menu entry.

Integrated titlebar (32px strip, 46px caption buttons):

- Sits transparently directly over the full-bleed warm `field` (`L0`), sharing its native frosted glass or solid fallback rather than drawing a separate opaque bar or system-colored strip.
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

The field fills the window. Insets are margins between panels (home/workbench 8px, narrow workbench windows 4px), not a band around a sheet. Home is a window-filling document workspace inside one material panel, keeping text protected from the native backdrop. The header contains the wordmark, Settings/Displays and Open PDF. At a viewport width of at least 1000px scaled by the saved UI font size, a 340px scaled recent list sits left of a flexible, dominant PDF preview with a 32px gap. Narrower windows stack history before preview. A compact output-configuration footer replaces the tall preparation well. Keep the existing home scroll and keyboard anchors; no marketing filler.

Home shows the configured output target and start mode, never claiming that configuration is observed live output. Settings/Displays stay directly accessible. Preserve the first-three recent limit, accurate local minute timestamps, full-path disclosure, original import paths and user font preferences. Row activation selects a read-only first-content-page preview; only Open this document imports it. Initially select the first recent document. Selection has a reserved check mark and warm accent wash. Clear old pixels immediately when selection changes, and show localized loading, empty, unavailable and retry states.

Normal projection status has no extra row below the top bar. Output failures and black/white cover recovery retain the accessible status row and Displays/Restore actions. The navigation group orders Previous, progress/Jump, Next. Progress emphasizes the logical current slide with 26px semibold tabular accent-text digits; the total and optional overlay detail are secondary. Reserve current-digit width from the total's digit count, preserve the complete accessible logical/step/physical detail, and update immediately without animation. Primary orange fill remains reserved for primary actions.

The workbench is a top bar panel, one panel per pane (Current, Next, Notes) and a bottom bar panel. Panes inset their picture 4px horizontally; captions and pictures carry the vertical inset, so saved split ratios still describe pane heights exactly. Notes are a well inside their panel. Statuses between panels (crop check, shell warnings, unsaved settings) are notice panels led by a status-colored dot. Settings, Displays and the crop check fill one full-height panel.

With 4px panel padding, an inner rounded area uses a 12px radius, because outer radius = inner radius + padding.

Inside the full-height Settings panel, the navigation and category body are centered together. Each category has a square-cornered full-row target. Its text box is centered on both axes. The selected row has an accent wash and an absolutely positioned short left bar. Resting rows expose the panel, hover and press use control fills, and the focus ring follows the rectangle. The 176px navigation rail sits 16px from the body. The body is at most 960px wide, or 1120px for Displays. The title starts at the navigation edge, Close ends at the body edge, and save feedback starts under the body. Below 760 logical pixels the navigation wraps above the scrolling body.

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
  background: linear-gradient(180deg, rgba(255,255,255,0.21), rgba(255,250,244,0.21) 55%);
  border-radius: 16px;
}
.panel.is-solid {
  background: linear-gradient(180deg, #FFFDF9, #FBF8F4 55%);
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
| `{rounded.readout}` | 6px | Wall clock and presentation timer |
| `{rounded.well}` | 12px | Wells |
| `{rounded.panel}` | 16px | Panels and the home card |
| `{rounded.floating}` | 16px | Menus and dialogs |
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
| Selected segment | `segment` | ink at 0.03 / 0.06 over `segment` | `ink` |
| Disabled | `control` | none | `muted` |
| Titlebar control (min/max) | none (over `field`) | `titlebar-control-hover` / `titlebar-control-pressed` | `titlebar-icon` (inactive: `titlebar-inactive-icon`) |
| Titlebar close | none (over `field`) | `titlebar-close-hover` / `titlebar-close-pressed` | `titlebar-icon` (inactive: `titlebar-inactive-icon`; hover/pressed: `titlebar-on-close`) |

The integrated presenter titlebar is 32px tall (`TITLEBAR_HEIGHT`), sits transparently on `field`, and places square-edged 46px-wide caption buttons (`TITLEBAR_BUTTON_WIDTH`) on the trailing edge (hidden on macOS, where a 78px leading inset reserves space for native traffic lights positioned at `(12px, 10px)`). Active and unfocused windows switch between `titlebar-ink`/`titlebar-icon` and `titlebar-inactive-ink`/`titlebar-inactive-icon`. Clicking Close routes through the presenter's safe close chain; danger red (`titlebar-close-hover` / `titlebar-close-pressed`) is strictly confined to the Close hover and pressed states.

Grouped choices (black/white cover, drawing tools, crop layouts, setting choices, page-browser numbering, view and grid density) use a segmented control: one `control` track, the selected option lifted to `segment`. Each option's 2px focus ring and 2px gap form the 4px inset between track and segment. Do not wrap a group in a colored shell.

Selection never relies on color alone and never inserts a glyph that changes the label width. Apart from settings category rows, a choice that can be selected reserves an 8px indicator slot; the selected option fills it with a `focus` dot (`on-primary` on an orange fill). The black/white swatch is its own slot and gains a 2px `focus` ring when selected. Menu rows keep one reserved check column with the same dot. Toggle buttons expose their state as accessible toggled state.

Keyboard focus is a 2px `{colors.focus}` ring outside a 2px gap.

## Accessibility

Meet WCAG 2.2 AA on the worst specified background. The native panel is measured as the panel layer over the field over a pure black and a pure white desktop; the table lists the lowest of those composites. Text is at least 4.5:1, the focus ring and selection marks at least 3:1. A labeled button is identified by its text, so it does not need a 3:1 outline.

The table uses the rounded 8-bit composites checked by the theme tests. It includes both field and panel gradient ends, black and white desktops, opaque fallback stops, standard source-over and Windows Direct3D additive quad alpha. Integrated titlebar tokens are evaluated directly against the solid `field` endpoints and the worst-case black/white desktop composites of the translucent field (and, for titlebar control overlays, both source-over and Windows Direct3D additive quad alpha).

| 前景 | 背景 | 浅色 | 深色 | 用途 |
| --- | --- | --- | --- | --- |
| `ink` | `layer_solid` | 16.16 (`#1F1B17` / `#FBF8F4`) | 13.34 (`#F5F1EB` / `#2A2622`) | 正文，实色面板 |
| `ink` | 毛玻璃合成 | 8.67 (`#1F1B17` / `#BDB8B0`) | 8.05 (`#F5F1EB` / `#4C4844`) | 正文，毛玻璃面板最差合成 |
| `body` | `layer_solid` | 10.95 (`#3D3833` / `#FBF8F4`) | 10.94 (`#E2DBD2` / `#2A2622`) | 次级正文 |
| `muted` | 毛玻璃合成 | 4.95 (`#49433C` / `#BDB8B0`) | 4.83 (`#C4BCB2` / `#4C4844`) | 次要文字，毛玻璃面板最差合成 |
| `body` | 毛玻璃合成 | 5.88 (`#3D3833` / `#BDB8B0`) | 6.60 (`#E2DBD2` / `#4C4844`) | 文字或状态标记，最差合成 |
| `accent_text` | 毛玻璃合成 | 4.67 (`#76341C` / `#BDB8B0`) | 4.57 (`#EBA98C` / `#4C4844`) | 文字或状态标记，最差合成 |
| `success` | 毛玻璃合成 | 4.65 (`#215132` / `#BDB8B0`) | 4.95 (`#8DCEA0` / `#4C4844`) | 文字或状态标记，最差合成 |
| `warning` | 毛玻璃合成 | 4.62 (`#66400A` / `#BDB8B0`) | 4.94 (`#E8B774` / `#4C4844`) | 文字或状态标记，最差合成 |
| `error` | 毛玻璃合成 | 4.75 (`#802821` / `#BDB8B0`) | 4.81 (`#FFA5A0` / `#4C4844`) | 文字或状态标记，最差合成 |
| `muted` | `control_pressed` | 6.94 (`#49433C` / `#DCD9D5`) | 4.77 (`#C4BCB2` / `#4C4945`) | 按下的中性控件上的次要文字 |
| `on_accent` | `accent` | 5.65 (`#1F1712` / `#D97757`) | 5.65 (`#1F1712` / `#D97757`) | 主按钮文字 |
| `on_accent` | `accent_pressed` | 4.71 (`#1F1712` / `#C9694A`) | 4.71 (`#1F1712` / `#C9694A`) | 主按钮按下 |
| #FFFFFF | `accent` | 3.12 (`#FFFFFF` / `#D97757`) | 3.12 (`#FFFFFF` / `#D97757`) | 白字压橙色，禁止 |
| `accent_text` | `accent_wash` | 7.60 (`#76341C` / `#F7E6DC`) | 6.90 (`#EBA98C` / `#3A2A22`) | 橙色文字压选中洗底 |
| `accent_text` | `control` | 7.76 (`#76341C` / `#EEEBE7`) | 5.94 (`#EBA98C` / `#3B3734`) | 橙色文字压中性控件 |
| `focus` | 毛玻璃合成 | 4.67 (`#76341C` / `#BDB8B0`) | 4.57 (`#EBA98C` / `#4C4844`) | 焦点环与选中标记，毛玻璃面板最差合成 |
| `focus` | `control_pressed` | 6.55 (`#76341C` / `#DCD9D5`) | 4.51 (`#EBA98C` / `#4C4945`) | 焦点环压最深中性填充 |
| `focus` | `segment` | 9.22 (`#76341C` / `#FFFFFF`) | 4.21 (`#EBA98C` / `#524D48`) | 选中分段上的选中标记 |
| `ink` | `segment` | 17.11 (`#1F1B17` / `#FFFFFF`) | 7.42 (`#F5F1EB` / `#524D48`) | 选中分段文字 |
| `border` | `well` | 3.19 (`#8C847A` / `#F3EEE7`) | 4.63 (`#8F877C` / `#221F1C`) | 输入框边界 |
| `error` | `error_wash` | 7.60 (`#802821` / `#F8E3DF`) | 7.55 (`#FFA5A0` / `#3B2523`) | 停止/危险文字 |
| `warning` | `layer_solid` | 8.61 (`#66400A` / `#FBF8F4`) | 8.19 (`#E8B774` / `#2A2622`) | 警告文字 |
| `titlebar_ink` | 实色底场（`field_bottom`） | 13.86 (`#1F1B17` / `#EFE6DB`) | 14.03 (`#F5F1EB` / `#26221E`) | 一体化标题栏激活标题，实色底场 |
| `titlebar_ink` | 底场毛玻璃最差合成 | 7.09 (`#1F1B17` / `#ACA69E`) | 8.98 (`#F5F1EB` / `#45413E`) | 一体化标题栏激活标题，毛玻璃最差合成 |
| `titlebar_icon` | 实色底场（`field_bottom`） | 13.86 (`#1F1B17` / `#EFE6DB`) | 14.03 (`#F5F1EB` / `#26221E`) | 一体化标题栏激活控制图标，实色静止态 |
| `titlebar_icon` | 底场毛玻璃最差合成 | 7.09 (`#1F1B17` / `#ACA69E`) | 8.98 (`#F5F1EB` / `#45413E`) | 一体化标题栏激活控制图标，毛玻璃静止态 |
| `titlebar_inactive_ink` / `titlebar_inactive_icon` | 实色底场（`field_bottom`） | 6.95 (`#524B44` / `#EFE6DB`) | 6.41 (`#ACA49A` / `#26221E`) | 一体化标题栏失焦标题与图标，实色底场 |
| `titlebar_inactive_ink` / `titlebar_inactive_icon` | 底场毛玻璃最差合成 | 3.55 (`#524B44` / `#ACA69E`) | 4.10 (`#ACA49A` / `#45413E`) | 一体化标题栏失焦标题与图标，毛玻璃最差合成 |
| `titlebar_icon` | `titlebar_control_hover` 最差合成 | 6.21 (`#1F1B17` / `#A19B93`) | 6.90 (`#F5F1EB` / `#55524F`) | 一体化标题栏激活控制按钮悬停 |
| `titlebar_icon` | `titlebar_control_pressed` 最差合成 | 5.56 (`#1F1B17` / `#98928B`) | 5.81 (`#F5F1EB` / `#605D5B`) | 一体化标题栏激活控制按钮按下 |
| `titlebar_inactive_icon` | `titlebar_control_hover` 最差合成 | 3.11 (`#524B44` / `#A19B93`) | 3.15 (`#ACA49A` / `#55524F`) | 一体化标题栏失焦控制按钮悬停 |
| `titlebar_on_close` | `titlebar_close_hover` | 5.57 (`#FFFDF9` / `#C42B1C`) | 5.57 (`#FFFDF9` / `#C42B1C`) | 一体化标题栏关闭按钮悬停 |
| `titlebar_on_close` | `titlebar_close_pressed` | 7.19 (`#FFFDF9` / `#A62317`) | 7.19 (`#FFFDF9` / `#A62317`) | 一体化标题栏关闭按钮按下 |
| `titlebar_ink` | `titlebar_inactive_ink` | 1.99 (`#1F1B17` / `#524B44`) | 2.19 (`#F5F1EB` / `#ACA49A`) | 一体化标题栏激活与失焦视觉区分（≥ 1.8） |

## Writing Tone

State the status, then the number, then the action. Keep labels short. No slogan and no marketing headline. Do not tighten Chinese text to match the Latin number tracking.

## Do's and Don'ts

### Do

- Put presenter text and controls only on a panel, a well, a control or a raised surface; only the 32px integrated titlebar sits transparently on the window field using its dedicated field-calibrated `titlebar-*` tokens.
- Use `{colors.primary}` with `{colors.on-primary}` for the one primary action in a view.
- Use `{colors.focus}` for the 2px focus ring and selection marks.
- Use `{colors.titlebar-close-hover}` and `{colors.titlebar-close-pressed}` with `{colors.titlebar-on-close}` exclusively for the integrated titlebar Close button's hover and press states.
- Separate layers by opacity and lightness of one warm hue.
- Use the solid fallback below 960px, for reduced transparency, on Windows 10 (build < 22000 or unresolved build), and when the platform cannot blur.
- Set numbers in `tabular-nums`.

### Don't

- Except for the integrated titlebar (whose tokens are calibrated against worst-case black/white desktop composites), don't put text or controls directly on the bare field. Over native blur the field shows an arbitrary desktop.
- Don't indent a whole-window sheet or draw an outer window stroke.
- Don't outline every button. Use `border` only for inputs and frames that need a visible edge.
- Don't add a second accent hue. No green or purple in presenter chrome; status colors stay in status text and marks, and titlebar close danger red (`#C42B1C` / `#A62317`) never appears on general buttons or panel fills.
- Don't put white text on `#D97757`.
- Keep the window field warm. At the specified opacity the desktop contributes 28% to the light field and 14% to the dark field before applying a panel.
- Don't mark selection with a prefixed glyph that changes button width.
- Don't wrap a group of buttons in a colored shell.
- Don't use Inter, Plus Jakarta Sans, or DM Sans.
- Don't apply negative letter-spacing to Chinese running text.
- Don't add a second shadow, a glow, or a neon gradient.

## Responsive Behavior

| Condition | Field | Panel |
| --- | --- | --- |
| 960px and above, glass on, Windows 11+ (build ≥ 22000) or non-Windows host, system allows blur | Gradient at 0.72 (dark 0.86) over requested blur | Highlight to base at 0.21 (dark 0.07) |
| Windows 10 (build < 22000 or unresolved build) with glass on | Opaque gradient | `layer-solid-top` to `layer-solid` |
| Below 960px | Opaque gradient | `layer-solid-top` to `layer-solid` |
| Reduced transparency, unsupported backend or glass off | Opaque gradient | `layer-solid-top` to `layer-solid` |

Pills stay pills. The focus ring stays 2px. Keyboard focus does not move or scale.

## Known Gaps

- No chart palette.
- Contrast is measured on the specified surfaces and on the native composites over black and white desktops, not on photographs behind the window.
- If Figtree is unavailable, fall back to the system sans-serif and keep `"PingFang SC", "Noto Sans SC"` after the Latin family.
- Title weight 650 needs the Figtree variable font. If only a static file is available, use 600.
- PDF previews are still painted with square corners. The paint path is the slide surface shared with the audience window, which this restyle does not change.

## FerroStage native presenter

The presenter owns this theme. Its integrated titlebar, home, document header, timer, tools, settings, menus and dialogs use the single Rust token layer in `crates/ferrostage-ui/src/theme.rs`. PDF previews and private note images retain the document's colors and aspect ratio. The audience window paints only its public snapshot, including its original black and white covers, and never reads this theme.

**Integrated Titlebar.** Home, Settings/Displays and the Ready/Presenting workbench share a 32px-tall transparent integrated titlebar painted directly over the full-bleed warm `field`, always matching FerroStage's Light or Dark palette regardless of OS appearance. Trailing 46px caption buttons provide minimize, maximize/restore (preserving native drag, double-click zoom and Windows Snap Layouts) and safe close; macOS reserves a 78px leading inset for native traffic lights (`(12px, 10px)`) without duplicate controls, while Linux follows platform decoration negotiation. Unfocused windows lower title and icon emphasis from `titlebar-ink`/`titlebar-icon` to `titlebar-inactive-ink`/`titlebar-inactive-icon`, and fullscreen presenter mode hides the titlebar and restores it on exit. Titlebar Close danger red (`#C42B1C` / `#A62317`) is confined solely to the Close button's hover and pressed states.

**Appearance.** General settings provide Light, Dark and System plus an independent glass preference. The initial appearance is light with glass enabled. An explicitly stored theme survives upgrade. Applying appearance settings updates the presenter in place through the existing settings transaction. Selecting a solid surface keeps the chosen color scheme. On Windows 10 (build < 22000 or when the Windows build number cannot be resolved), the presenter automatically falls back to the opaque solid surface to eliminate DWM window-drag and resize stutter, skips periodic system transparency polling, preserves the saved `glass_effect` preference unchanged, and reports the fallback honestly under General → Frosted glass; native blur remains active only on Windows 11 and above (build ≥ 22000) or non-Windows hosts when enabled, at least 960 logical pixels wide, and permitted by system transparency settings. A window below 960 logical pixels, a system reduced-transparency preference or an unsupported backend likewise uses the solid fallback while keeping the saved glass choice.

**Material.** Maintained GPUI has no per-element backdrop blur, only window-level `WindowBackgroundAppearance::Blurred`. When glass is available, the field and panel share an opacity budget. At a blank panel point the theoretical desktop weight is 22.12% in light and 13.02% in dark with source-over, or 7% in both themes with the maintained Windows quad blend. The system controls the blur. Settings reports the request. Opaque fallback fills preserve the upper highlight. Panels are not outlined; the focus indicator and control fill provide the interactive boundary. Font overrides and interface scaling apply to presenter controls.

**Composition.** Home uses the window-filling recent-list/selected-preview workspace described above, with Open PDF, Settings/Displays and a compact output summary. It scrolls on small windows. Ready and Presenting group the optional local HH:MM:SS clock and small dial on a 6px-radius inset before the filename. The centered presentation group stacks a short status above 30px tabular digits, aligned left on a shallow glass rectangle. Its upper highlight uses `raised` at 0.32 opacity and fades to `layer-solid-top` at 0.04 over the panel. Both stops become opaque in fallback mode. Its numeric column reserves a signed three-digit minute reading in the chosen font. Fresh digits use ink, running digits use accent text, and the configured thresholds use warning and error. A ring identifies pause and a square identifies a stopped session. Pause, Resume and Reset sit beside the readout. Under 800px the timer group occupies its own centered row; controls wrap when space runs out. Standard layout keeps Current on the left and Next and Notes on the right. Notes-first keeps Notes on the left at widths of 800 logical pixels and above. Saved splitter ratios, keyboard targets and note scrolling apply in both variants. Changes of appearance preserve the current page, timer, annotations, media and window roles.

**Controls.** Primary actions use the orange fill with dark text. Preserve hover, pressed, disabled, selected and focused states, the 2 px focus indicator and the minimum 44 px interactive target. Time digits keep a stable width. PDF and media are never assigned control fonts or tinted by the presenter.

The presenter uses an installed interface-font override first, then installed Figtree, then the platform UI family. It reads the available family names once per session. The General settings font button opens a focused, single-line search field. Typing narrows installed family names, arrows browse the scrollable results, and Enter or a click applies the highlighted or clicked family. The panel shows the current font separately. Escape returns to the button, Tab leaves the search, and Use default restores the system choice. The clock and timer request tabular digits on that same face, so a missing optional font does not trigger a new native font search when the first PDF opens.
