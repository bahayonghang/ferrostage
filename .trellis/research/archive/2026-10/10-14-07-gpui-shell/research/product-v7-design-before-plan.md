---
version: alpha
name: FerroStage
description: Native presenter UI inspired by the Claude design language.
colors:
  primary: "#A34F35"
  primary-hover: "#93452F"
  primary-active: "#823C29"
  on-primary: "#FFFFFF"
  canvas: "#FAF9F5"
  surface-soft: "#F5F0E8"
  surface-card: "#EFE9DE"
  ink: "#141413"
  body: "#3D3D3A"
  muted: "#69665F"
  border: "#858078"
  hairline: "#E6DFD8"
  focus: "#A34F35"
  success: "#28613D"
  warning: "#825315"
  error: "#A63232"
  canvas-dark: "#181715"
  surface-soft-dark: "#1F1E1B"
  surface-card-dark: "#252320"
  ink-dark: "#FAF9F5"
  body-dark: "#E6DFD8"
  muted-dark: "#B9B3A8"
  border-dark: "#91897C"
  focus-dark: "#E5A38B"
  success-dark: "#8DCEA0"
  warning-dark: "#E8B774"
  error-dark: "#FFA5A0"
  audience-curtain: "#000000"
typography:
  display:
    fontFamily: "Georgia, SimSun, serif"
    fontSize: 28px
    fontWeight: 400
  title:
    fontFamily: "Segoe UI, Microsoft YaHei UI, system-ui"
    fontSize: 20px
    fontWeight: 600
  body:
    fontFamily: "Segoe UI, Microsoft YaHei UI, system-ui"
    fontSize: 16px
    fontWeight: 400
  label:
    fontFamily: "Segoe UI, Microsoft YaHei UI, system-ui"
    fontSize: 14px
    fontWeight: 500
  timer:
    fontFamily: "Consolas, ui-monospace"
    fontSize: 24px
    fontWeight: 500
rounded:
  sm: 4px
  md: 8px
  lg: 12px
spacing:
  xxs: 4px
  xs: 8px
  sm: 12px
  md: 16px
  lg: 24px
  xl: 32px
sizes:
  control: 44px
  focus-width: 2px
  focus-gap: 2px
  toolbar: 64px
  sidebar: 208px
---

<!-- design-md: entry=brand:claude; source=https://github.com/VoltAgent/awesome-design-md@3b60532; license=MIT -->
<!-- customized: FerroStage native presenter; accessible clay actions; light/dark palettes; system fonts; task-owned runtime verification. -->

## Overview

Warm cream surfaces, dark ink, and a restrained clay primary action keep the presenter's attention on the slides. This is a FerroStage interface inspired by Claude's design language. Product UI uses FerroStage identity and existing project assets only.

The single Rust token layer belongs to task 07 at `crates/ferrostage-ui/src/theme.rs`. Tasks 08 and 09 extend that layer. Theme resolution consumes `general.theme` (`light`, `dark`, `system`); system mode responds to OS theme changes. Dark mode maps each surface/text/semantic role to its `-dark` token; the primary button family stays the same. No new settings keys or parallel palettes.

## Colors

Light canvas/surfaces use ink, body, or muted text. Dark surfaces use their dark counterparts. Primary actions use white on primary/hover/active. Semantic text uses matching light/dark success, warning, or error tokens and an explicit label/icon. Hairlines are decorative separators only; interactive boundaries use border. Keyboard focus uses the matching focus token, with a surface-colored gap around primary buttons.

PDF pixels, rasterized speaker notes, media, annotation colors, and black/white audience overlays are content; never tint them or apply a theme filter. The audience view remains snapshot-only and receives no settings or theme controller. Its curtain is pure black and has no controls.

## Typography

Use installed system fonts; never download proprietary brand fonts. Georgia is reserved for the Idle welcome title and short empty-state headings. Controls, filenames, and settings use the sans stack; elapsed time uses the monospace stack. Font fallback must cover Chinese. Text scaling must not clip labels or hide Start/Stop. Long filenames truncate in the header and expose the full name on demand.

## Layout

07: compact header with filename and textual stage, a dominant Open PDF action in Idle, and Start/Stop in Ready/Presenting. Show the reason beside a disabled Start button. Keep import errors and topology warnings in the presenter.

08: current slide on the left, next slide above notes on the right; header holds clock/timer, bottom toolbar holds navigation and tools. Respect persisted split ratios. Use quiet panel labels and thin decorative dividers; selected tools have a border and textual/accessibility state. Keep slide aspect ratio and letterboxing. Thumbnails stay subordinate to the current slide.

09: category navigation on the left and a scrollable settings panel on the right. Display topology is the primary content of the Displays category. Keep saved/unsaved/read-only state and Retry visible. Use short descriptions such as “Saved audience display is disconnected” and “Language changes after restart”.

## Elevation & Depth

Prefer surface changes and borders to shadows. Only presenter-owned modal/popover layers may use subtle elevation. No gradients, glow, decorative dashboards, or extra feature cards.

## Shapes

Controls use md radius, panels lg, status badges sm. Content images retain their natural rectangle. All interactive targets are at least 44 by 44 logical pixels, including tool icons.

## Components

**button-primary**: label plus optional progress indicator, primary/on-primary, label type, control height, xs/md padding, md radius. Hover uses primary-hover; pressed uses primary-active; focus adds the focus ring; disabled uses surface-card/muted with explanatory text and no action; loading preserves geometry and blocks duplicate invocation; error displays text and retry. Dark primary controls also use border-dark to separate the clay fill from elevated surfaces.

**button-secondary / tool-button**: surface-soft/body with border, control height and md radius. Hover uses surface-card; selected adds border plus explicit selected state. Disabled, focus, loading, and error follow the primary control contract.

**field / select / slider / keybinding-row**: label above or beside the control, description below, surface-soft/body and border. Hover uses surface-card; focus uses focus; invalid uses error plus a specific message. Read-only future-version controls cannot mutate settings; save failure leaves effective session values visible with Retry. Keyboard conflicts are named, not color-only. Native accessibility names and values are mandatory where GPUI supports them; unsupported behavior remains an explicit gap.

**status / timer**: stage text always accompanies color. Timer warnings use semantic text with a label/icon, retain legibility in both themes, and read core elapsed time. No flashing or time-dependent motion as the sole signal.

**modal / category-nav**: presenter-owned; Escape closes the topmost modal and consumes the event. Tab/Shift+Tab follow visible order, Enter/Space activate; restore focus to the invoker. Settings navigation marks selection with text weight and a border as well as a surface change.

## Do's and Don'ts

- Use the single token layer; do not scatter raw styling values across views.
- Keep Open, Start, Stop, and Retry labels explicit; do not add speculative actions.
- Preserve read-only PDF and notes isolation; do not send presenter theme/state to the audience.
- Honor reduced motion; use no nonessential animation in the initial native UI.
- Use FerroStage branding; do not reproduce Claude logos, product names, artwork, or proprietary fonts.

## Responsive Behavior

| Logical width | Native window behavior |
| --- | --- |
| >= 1280 | Full workbench split and labeled toolbar; settings category rail |
| 800–1279 | Compact secondary labels; wrap tool groups; settings keeps a scrollable panel |
| 390–799 | Stack previews; notes remain scrollable; categories collapse to a keyboard-accessible selector; Start/Stop remain visible |
| 320–389 | Same compact layout; only internal content scrolls; controls retain target size |

Desktop is the target platform; narrow checks are resizable native-window checks, not a mobile product. Verify 1440x900, 800x600, 390x844, and 320x640 logical sizes plus mixed DPI. If the native framework enforces a larger minimum window, document the actual minimum and mark smaller probes NOT_RUN with that reason; do not claim narrow acceptance or add a mobile product to this task. Supported desktop sizes and all visible controls must still pass.

## Iteration Guide

1. Change this document before the token layer when design decisions change.
2. 07 owns foundational components and all control states; 08 owns workbench composition; 09 owns theme/system and settings interactions.
3. Compare native screenshots in both themes against this source. Browser mockups cannot establish GPUI behavior.
4. Run contrast checks for every allowed text/surface, boundary, and focus pair. Normal text >= 4.5:1, large text and UI/focus >= 3:1.
5. Record keyboard, hover, disabled, loading/error, long-label, and reduced-motion checks independently; do not infer visual success from compile tests.

## Known Gaps

- The upstream coral/white 14px button pair is insufficient; primary is darkened to #A34F35. Muted text and semantic colors are also adapted for contrast. Original display fonts are replaced by installed system fonts.
- Task07 has a native GPUI shell and shared theme. Light/dark/system initial appearance, Chinese/en-US labels, long filename truncation, Start/Stop, screen Swap and close have been observed. The earlier focused Enter/Space, disappearing-control focus and stale save-feedback defects are repaired and natively retested through v6. Native chooser import and saved-role relaunch pass. Some narrow captures exist; the exact logical-client size/state matrix, live OS theme-change event, drag/drop, physical topology, real PDF workbench and final settings UI acceptance remain pending. Current source-bound evidence is recorded in task07 research/native-v6-checkpoint.md and native-v5-checkpoint.md; these partial checks do not establish later-task acceptance.
- GPUI accessibility and theme-change APIs require the pinned-version research gate in 07. An unsupported API must be recorded, not fabricated.

## QA

- [ ] Single Rust theme layer and component states conform to this document.
- [ ] Contrast passes for all used pairs in light and dark modes.
- [ ] Native keyboard navigation, focus restoration, and state feedback pass.
- [ ] Native screenshots cover the listed sizes, both themes, and mixed DPI.
- [ ] No brand assets/proprietary fonts or tinted PDF/audience content.
