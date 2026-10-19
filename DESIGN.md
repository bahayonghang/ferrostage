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
  home-brand:
    fontFamily: "Georgia, serif"
    fontSize: 32px
    lineHeight: 38px
    fontWeight: 600
  display:
    fontFamily: "Segoe UI, Microsoft YaHei UI, system-ui"
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
    fontSize: 28px
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
  home-content-max: 760px
  control: 44px
  focus-width: 2px
  focus-gap: 2px
  toolbar: 64px
  session-control: 56px
  presenting-session-control: 44px
  session-icon: 26px
  session-width-at-default-text-size: 208px
  sidebar: 208px
---

<!-- design-md: entry=brand:claude; source=https://github.com/VoltAgent/awesome-design-md@3b60532; license=MIT -->
<!-- customized: FerroStage native presenter; accessible clay actions; light/dark palettes; system fonts; task-owned runtime verification. -->
<!-- customized: home-design-refactor, dedicated FerroStage wordmark, compact file entry, Idle-only information hierarchy. -->

## Overview

Warm cream surfaces, dark ink, and a restrained clay primary action keep the presenter's attention on the slides. This is a FerroStage interface inspired by Claude's design language. Product UI uses FerroStage identity and existing project assets only.

The single Rust token layer belongs to task 07 at `crates/ferrostage-ui/src/theme.rs`. Tasks 08 and 09 extend that layer. Theme resolution consumes `general.theme` (`light`, `dark`, `system`); system mode responds to OS theme changes. Dark mode maps each surface/text/semantic role to its `-dark` token; the primary button family stays the same. Theme resolution introduces no additional theme keys or parallel palettes.

## Colors

Light canvas/surfaces use ink, body, or muted text. Dark surfaces use their dark counterparts. Primary actions use white on primary/hover/active. Semantic text uses matching light/dark success, warning, or error tokens and an explicit label/icon. Hairlines are decorative separators only; interactive boundaries use border. Keyboard focus uses the matching focus token, with a surface-colored gap around primary buttons.

PDF pixels, rasterized speaker notes, media, annotation colors, and black/white audience overlays are content; never tint them or apply a theme filter. The audience view remains snapshot-only and receives no settings or theme controller. Its curtain is pure black and has no controls.

## Typography

Use installed system fonts; never download proprietary brand fonts. Idle uses a dedicated FerroStage wordmark in Georgia, 32/38px semibold, separate from ordinary headings. All other home copy uses the sans stack: instruction 14/22px, actions and filenames 14/20px, recent section and parent paths 12/18px. Do not apply Latin tracking to Chinese. Other short empty-state headings may use the display stack. Controls, filenames, and settings use the sans stack; elapsed time uses the monospace stack. Font fallback must cover Chinese. Text scaling must not clip labels or hide Start/Stop. Long filenames truncate in the header and expose the full name on demand.

Optional schema-v1 leaves `general.ui_font_family = ""` and `general.ui_font_size = 16` control interface typography. Empty family means System default; installed families are selected in a bounded searchable list. Missing families retain the requested setting and show a fallback notice. Base size accepts 12–24 logical pixels; existing text roles scale by base/16. The home wordmark keeps its fixed Georgia role, timers keep their monospace face and scale, and OS titlebar, PDF/raster notes, media and audience pixels are excluded. A mixed Chinese/Latin preview and explicit default restoration accompany these preferences.

## Layout

Idle: one app-content FerroStage wordmark, quiet Settings/Displays utilities, one short instruction, one natural-width Open PDF action, then up to three recent-file rows. Native Idle title is `FerroStage`. Omit document-status chrome, clocks/timer, duplicate Open and disabled Start footer in Idle. Preserve import/topology warnings and unsaved/save-error Retry. Ready/Presenting retain filename/stage header and Start/Stop; show the reason beside a disabled Start button there. Visibility changes never mutate timer/settings state.

Home content is a top-anchored, centered column capped at 760px. Use 32px side gutters at widths >=800px, 16px below; top inset 48px at heights >=600px, 24px below. Brand, copy, action and recent text share a left edge. Utilities wrap into a separate row below 600px width. Section spacing uses 8/16/24/32px. The home scroll container includes conditional feedback and remains usable at 320x240 logical client size. No large welcome card or fixed bottom action strip.

08: current slide on the left, Next upper-right and original Notes lower-right. Fresh defaults reserve 40% width for the right column and 65% of its pane height for Notes; the effective Notes share is clamped to 55–80%. Existing saved ratios remain authoritative. The horizontal divider measures the complement of the upper Next pane after excluding its bar and gaps; dragging down makes Next taller and Notes shorter. Each Notes raster fills the inner width at its natural aspect height, without flex shrinking; multiple pages remain ordered in one vertical scroll. Notes stay original PDF pixels, with source typography authored outside the app and no text reflow or mode/font selector. Header begins with a quiet 14px monospace local clock, then filename and session label. A labeled 28px monospace presentation readout remains before utility actions, using neutral ink normally and semantic warning colors at existing thresholds. It has no separate badge background or animation; full signed values fit without truncation. One bottom strip groups Start/Stop, Previous/Next and compact actionable page progress, followed by blanking, laser and secondary utilities. Presenting places Home/Open/Settings under More and Pen/Highlighter/Eraser/Clear under Drawing; display access, Page browser and Layout stay direct. Ready retains management controls. Full logical/step/physical progress is available on hover, keyboard focus and accessibility. No separate full-width session footer or leftover divider. Disabled Start explains its reason locally; busy preparation preserves its geometry and blocks duplicate activation. Keep slide aspect ratio and letterboxing. Recent files belong only to Idle. Progress, Jump and Page browser share one contextual panel above the toolbar, retaining the workbench underneath. The bordered number field supports caret/selection and shows the active Content or PDF original numbering context; edits only locate a target. Explicit Enter/Go or whole-thumbnail activation jumps directly once and closes. Independent 1–4 column/row controls default to 4×4 and update the visible group without navigation. Thumbnails appear only while this browser is open; the legacy show_thumbnail_strip setting is read-compatible but ignored. The elapsed talk timer never changes slides.

Start requests fullscreen on the existing presenter window and commits only after native presenter and audience placement readback. Stop or failed Start retires audience output and restores the captured normal/maximized/fullscreen state on the same window and monitor. Windows fullscreen toggles are asynchronous: restoration waits for entry before issuing one exit, and remains busy until verified or safely recovered. Startup fullscreen preferences and saved monitor roles are unchanged. A replacement presenter never receives the retired window's saved bounds. Native fullscreen and standing readability need runtime evidence; flags and headless geometry alone do not establish them.

09: category navigation on the left and a scrollable settings panel on the right. Display topology is the primary content of the Displays category. Keep saved/unsaved/read-only state and Retry visible. Use short descriptions such as “Saved audience display is disconnected” and “Language changes after restart”.

Settings use quiet category navigation and sans headings, a left-aligned content column capped at 960 logical pixels (1120 for Displays), section hairlines and content-driven rows of at least 56px. Labels and local help sit left, bounded controls right; no background card per field. General groups Appearance, Startup and Recent files. Recent files retain order and show filename plus real parent/full-path context with count/limit and a content-sized Clear. Keep all seven categories and existing edit/reset/recording behavior. Preference save status does not imply display allocation has been applied.

## Elevation & Depth

Prefer surface changes and borders to shadows. Only presenter-owned modal/popover layers may use subtle elevation. No gradients, glow, decorative dashboards, or extra feature cards.

## Shapes

Controls use md radius, panels lg, status badges sm. Content images retain their natural rectangle. All interactive targets are at least 44 by 44 logical pixels, including tool icons.

## Components

**button-primary**: label plus optional progress indicator, primary/on-primary, label type, control height, xs/md padding, md radius. Hover uses primary-hover; pressed uses primary-active; focus adds the focus ring; disabled uses surface-card/muted with explanatory text and no action; loading preserves geometry and blocks duplicate invocation; error displays text and retry. Dark primary controls also use border-dark to separate the clay fill from elevated surfaces.

**home-open / home-recent / home-utility**: reuse shared activation/focus/disabled/accessibility dispatch while keeping home faces distinct. Open retains the 44px face height plus existing focus reservation, with natural 120-200px face width for shipped labels at default scale. Recent rows form a flat table with a quiet header and separators: a flexible filename/actual-parent column and a shared right-aligned last-opened column. Retain 48-56px outer targets at default scale and allow content to grow with interface typography. Display persisted successful-open events as local `YYYY-MM-DD HH:MM`; legacy, invalid or unrepresentable times show localized unknown copy. Never derive dates from PDF metadata or filesystem access. Rows have quiet hover/pressed surfaces and visible keyboard focus, without permanent command-button outlines. Reveal complete paths on hover and keyboard focus and include the path and time in accessible labels. Same-named files retain distinguishable parent context. Settings/Displays keep explicit text and >=44px targets. Do not shrink global CONTROL to fix home width. Keep 0/1/3 recent states and existing busy/import guards; never invent dates, metadata or shortcuts.

**button-secondary / tool-button**: loaded-document header utilities are quiet text controls. Navigation uses a bounded neutral surface and regular 8px radius. Black/white audience toggles use small bordered black/white swatches beside explicit labels and a selected check. Drawing tools share a soft segmented group with regular 8px faces; selection uses a stronger surface and check, never color alone. Stop and Clear use semantic error text/border on the soft surface, with Clear separated from tool selection. Start retains the filled clay action. Ready Start uses a 56px face and scaled 208px minimum width; Presenting Stop uses the ordinary 44px minimum with natural width. Both retain a 26px geometric play/stop icon plus explicit label in the same group as Previous/Next. Every variant shares keyboard, accessibility, disabled and focus handling. Hover changes the surface; pointer press moves the face down 1px without animation. Text contrast is at least 4.5:1 and focus/necessary boundary contrast at least 3:1 in both themes.

**field / select / slider / keybinding-row**: label/help beside the control in ordinary rows, with content-driven multi-line variants. Choices share one soft segmented group and numeric minus/value/plus share one field group. Hover uses surface-card; focus uses focus; invalid uses error plus a specific message. Read-only future-version controls cannot mutate settings; save failure leaves effective session values visible with Retry. Keyboard conflicts are named, not color-only. Native accessibility names and values are mandatory where GPUI supports them; unsupported behavior remains an explicit gap.

**status / timer**: on document screens stage text always accompanies color. Presentation timing has a quiet inline Elapsed / Remaining / Overtime label, stable aligned digits and complete signed values; the independent current-time readout stays at the leading header edge. Remove the standalone Navigate tool button, keeping neutral navigation internally and reactivating the selected laser/pen/highlighter/eraser to exit it. Timer warning text precedes the fixed-minimum-width numeric readout so threshold changes do not move the digits or right-side controls. Warnings use semantic text with a label/icon, retain legibility in both themes, and read core elapsed time. Idle omits this presentation without changing data or preferences. No flashing or time-dependent motion as the sole signal.

**modal / category-nav**: presenter-owned; Escape closes the topmost modal and consumes the event. Tab/Shift+Tab follow visible order, Enter/Space activate; restore focus to the invoker. Settings navigation marks selection with a check, text weight and a soft surface, preserving the shared keyboard focus ring.

**page-browser**: one presenter-owned upward contextual panel, preferred 880px width with adaptive height above the toolbar trigger, constrained inside the window. Reuse 16px padding, 12px panel radius, 8px grid gap and existing 44px controls. Below title/status, Content pages / 内容页 (default) and PDF original pages / PDF 原页码 select the numbering used by input, range, targets, captions and group footer. Content numbers index the actual ordered content steps, excluding recognized notes; PDF numbers retain every original physical position. Never halve totals or infer odd/even pairing. Original-PDF notes positions are non-projectable text placeholders without note thumbnail requests. Entering a recognized notes number shows 备注页不可投影 and never redirects to content. Switching numbering converts a valid physical target or resets invalid input to the current page, renews interaction identity and never navigates, reconfigures the document, changes grouping, clears ink or saves settings. Remember the mode for the document session; new documents reset to Content. Explicit Enter/Go or whole-content-tile activation jumps once. Hover, cancelled presses, mode changes and group browsing are inert. Keep the existing visible-slot limit of 16, including notes placeholders; only content slots request thumbnails. Retire stale exact-key thumbnail work on mode changes and all thumbnail work on close. Input/helper/mode/density/footer remain stable while image content scrolls. Independent 1-4 column/row controls retain paired persistence, legacy custom counts, read-only and save-failure behavior. Escape/outside dismissal consumes the action and restores focus. Stop/Home/Swap/document replacement discard stale targets.

**chapter-directory**: extend the existing page-browser with Thumbnails / 缩略图 and Chapter directory / 章节目录 tabs below the common numeric controls/helper. Keep the 880px preferred contextual panel, theme tokens and typography. Thumbnails retain density/grid/group navigation; directory replaces those with one fully expanded two-level PDF-outline list. Use >=44px rows, 24px second-level indent, complete wrapped authored titles, a nonshrinking right-aligned Content / PDF number column, and quiet separators/surface steps instead of row cards. Preserve source order, unnumbered entries and duplicate destinations. Mark the committed current section with explicit text independently from focus/editor target, and reveal it on entering the directory. A valid title click or focused Enter/Space commits once through existing guarded navigation and closes; focus/scroll/view switches never navigate. Notes, unavailable and external targets stay disabled with a reason. Empty/partial states retain other navigation. View is document-session-only and independent from numbering, defaults to thumbnails for new documents, renews interaction identity on switch and retires invisible thumbnail work/cache. Contextual Escape/outside dismissal consumes input and restores focus. No collapse/search/new settings or decorative motion. Native acceptance is separate from the layout study.

**layout-navigation-menu**: retain the Layout & navigation / 布局与导航 trigger and existing upward, right-aligned 320px opaque contextual menu, 8px above its trigger with a 12px viewport margin. Its three rows are Automatic and No split radio choices, then Identify independent notes pages as a checkbox. Remove directional crop choices and the misleading physical-navigation checkbox; browser numbering is owned by the jump panel and does not alter the internal overlay-grouping flag. New documents default to No split with independent-note recognition enabled. Automatic is explicit opt-in and still describes detected directional layouts through a complete label mapping independent of selectable rows. Same-document reload retains overrides; new documents reset. Preserve localized document warnings. Selecting the effective requested choice closes without clearing ink or frames; actual document-option changes retain guarded reconfiguration, failure feedback and the annotation-loss notice. Arrow/Home/End/Tab traverse only visible rows; Enter/Space activates once; outside/Escape consumes dismissal and restores focus. Overrides remain session-only.

**display-assignment**: Settings and standalone Displays share one compact allocation body. Each host owns one scroll, without a fixed-height embedded scroller. Show count and Identify/Refresh first, then a contained 420×240px spatial overview beside numbered assignment rows in an ordinary window. Stack naturally when space is insufficient. Use one uniform contain scale, preserve desktop positions and portrait aspect, and keep number-linked callouts for small rectangles. The same projection supplies diagram, summary and rows: checked text means observed current application/PDF window; configured PDF is explicitly not running before Start; outlined text prefixed Pending means unapplied draft. Unknown native location stays unknown, never inferred from the requested target. Manual OS moves update observation only. No audience means no running PDF; Stop/Home retires its marker. Diagnostics collapse per display; missing/degraded/stale warnings and recovery remain visible. Apply retains native confirmation/rollback; allocation Cancel discards only its draft and never reverses autosaved fullscreen preferences. Ordinary preference Saved feedback does not imply allocation committed. Numbers match Identify, while settings bind MonitorId. Existing Swap keys remain secondary compatibility actions.

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

Desktop is the target platform. The workbench/settings refinement validates ordinary native-window core interactions and visuals, including relevant themes/locales and keyboard focus. Additional dimension, DPI and performance matrices are outside this task's user-approved acceptance scope; source/build checks do not establish native visual acceptance.

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
- [ ] Native screenshots and core interactions cover the ordinary-window scope and relevant themes/locales.
- [ ] No brand assets/proprietary fonts or tinted PDF/audience content.
