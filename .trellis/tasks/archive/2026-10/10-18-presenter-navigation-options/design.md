# Shared visual and interaction design

## Visual thesis and implementation strategy

An understated warm-dark presenter console, with slide imagery as the anchor and clay reserved for intentional actions. Preserve the existing light theme through semantic tokens. Use configured sans UI typography, Segoe UI and Microsoft YaHei UI fallback; the serif-like control treatment in screenshots is not the desired contract.

Content plan: orient with current logical/PDF progress, expose clear input, then visual destinations; layout choices stay beside their toolbar trigger.

Interaction thesis: no timed entry/exit or keyboard animation. Pointer presses use existing immediate 1px displacement. Focus, selected target and committed current page are distinct. A successful action updates the workbench and closes its surface.

Product strategy: native GPUI elements and the single Rust theme layer. No CSS/webview/new frontend framework. Task-local HTML is a review artifact only.

## 1. Theme and atmosphere

Keep current slide left, notes above next slide right, and existing toolbar grouping. Replace utility forms with a bounded page panel above progress and a smaller upward menu above Layout & navigation. No dimming scrim that conceals the result.

## 2. Palette and roles

| Role | Dark | Light |
| --- | --- | --- |
| Canvas | #181715 | #FAF9F5 |
| Field/menu soft | #1F1E1B | #F5F0E8 |
| Panel/hover | #252320 | #EFE9DE |
| Main text | #FAF9F5 | #141413 |
| Secondary text | #B9B3A8 | #69665F |
| Required boundary | #91897C | #858078 |
| Focus/current | #E5A38B | #A34F35 |
| Go action | #A34F35 with white | Same |

Read exact `theme.rs` tokens, including error roles. Do not tint PDF/notes/audience pixels. Normal text contrast >=4.5:1; essential boundary/focus >=3:1.

## 3. Typography

Base 16px configured sans; panel title 20px semibold, controls/labels 14px, secondary context 12px, all scaled with existing Typography. Page numeral uses current mono family at 20px. Chinese gets explicit fallback and no tracking. No font downloads.

## 4. Components and states

### Page browser

- Progress retains its position; grid action becomes `页面浏览`. Both plus existing Jump shortcut open one surface.
- Header: `跳转页面`, current logical/PDF context, explicit Close.
- Input: `PDF 页码`, bordered 120px field, `/ 190`, natural-width `跳转`. Height >=44px, caret/selection, 2px focus ring. Enter and Go share guarded submission.
- Reserve local helper space for target logical/step context or specific range/independent-notes error; do not move the input when errors appear.
- Density: `列` [1 2 3 4] `×` `行` [1 2 3 4], plus capacity. Selected values have a check and stronger neutral fill, not color alone.
- Grid: actual cropped slide thumbnails in equal cells, contain/letterbox, captions `PDF 69 · 第35页` and step where relevant. Committed page has `当前`; typed target and keyboard focus are separate indicators. Entire tile is one activation target, not two callbacks on image/caption.
- Footer: previous/next group and visible content range/count. Wheel moves one row, group controls one capacity. Last group is left aligned with empty positions, never duplicates.
- Loading/empty/error keep cell geometry and truthful feedback. No fake previews in product. No navigable content disables Go/group controls.

### Layout & navigation

Trigger: `布局与导航` / `Layout & navigation`, upward chevron when open. `演示选项` is too broad because tools/timer/displays live elsewhere. `视图选项` falsely suggests presenter-only effects, although these choices affect projected crop/mapping.

Group **备注布局**: 自动识别, 不拆分, 备注在左侧, 备注在右侧, 备注在上方, 备注在下方. Exactly one check reflects requested override. Auto shows secondary `当前：不拆分` or detected side, distinguishing requested mode from detected result.

Group **页面处理**: 识别独立备注页; 按 PDF 物理页导航. Independent check items, without true/false. Quiet helper explains metadata/overlay behavior. Physical navigation separates overlay steps; it does not expose excluded notes pages.

Whole rows activate. Toggles can be unchecked; checks reflect resulting state. Successful changes, including toggles, close immediately. Same-layout selection closes without reconfiguration. Failure leaves old checks and menu open with a local error. Remove raw `Layout None (Unsplit) · [] · []` from ordinary UI; retain real localized warnings in existing feedback.

Core reconfiguration clears strokes/laser. Preserve policy and show `更改后将清空批注` before activation when strokes exist. Dismissal never clears annotations. Audience updates through existing snapshots; do not promise presenter-only effects.

## 5. Layout

Starting targets in logical pixels, subject to normal-window review:

- Page panel preferred width 880 and initial height near 72% of viewport. Normal-window review showed that this height clips a rectangular 4x4 grid even after compacting chrome. Expand adaptively for the actual header/density/footer and row content, bounded by space above the toolbar trigger, 8px trigger gap and 12px viewport inset. Retain meaningful thumbnail height, not tiny image strips. Use 16px padding and 12px radius. Input/footer stay visible; legacy tall/custom or otherwise insufficient content may scroll internally.
- Layout menu preferred width 320, content-driven height, 12px radius, 8px trigger gap. Align right edge to trigger, grow upward, clamp inside viewport. Insufficient vertical space scrolls the menu body rather than replacing workbench.
- Targets >=44px, controls 8px radius, spacing 4/8/12/16/24px, grid gap 8px. Derive tile image height from available region, rows and captions; remove current fixed 135px height.
- No second toolbar/duplicate footer; keep Start/Stop reachable.

## 6. Depth

Opaque semantic surfaces with one boundary and optional minimal shared elevation. No blur/gradient/glow/heavy generic card shadows. Adjacent slide and notes content remains recognizable.

## 7. Interaction architecture

Use one presenter-owned contextual-surface state with PageBrowser and LayoutNavigation variants. These are no longer full-content Modal::Jump/Document render branches. Settings/Displays keep existing modal behavior. Shell owns edit state, validated transitions, grid anchor and reconfiguration; Presenter uses captured data, preserving synchronous first-frame borrow safety.

Opening one closes the other. Explicit Close, Escape, outside click, document replacement, Home, Stop and Swap dismiss consistently. Consume dismissal clicks, preventing drawing/jumping/tool activation beneath. Escape closes once without falling through to Stop. Restore invoking trigger focus on submit/cancel and do not steal audience focus. Pending native transactions disable contextual actions. Preserve deferred Shell dispatch.

The numeric field is a real focused editor, not a border around the global digit accumulator. Support selection/caret/replacement, Backspace/Delete, Home/End and plain-text paste. Invalid submission stays local. Use pinned GPUI input APIs, or a small focused numeric editor if necessary, recording unsupported accessibility behavior honestly; do not add a general editor framework.

## 8. Data and compatibility

- Interpret 1-based PDF input only after checked range + `slide_map.logical_for` validation. Valid target edits may reveal that target's browser group; they never stage/change committed projection. Invalid edits retain the last visible group.
- On open, reveal current page. On dimension changes preserve first visible step where possible. When document/map changes, clamp to new content map and discard stale edit errors/targets.
- Thumbnail identity includes generation plus map/crop epoch and step. Retire old queued/frame/texture work on group/dimension/map changes. Preserve single render worker and <=16 visible requests.
- Reuse schema-v1 keys: columns -> `grid_columns`, columns*rows -> `grid_visible_pages`. Update both atomically with one validated preference apply/save.
- Preserve valid nonrectangular old configs, e.g. 4 columns/13 pages, and tall legacy 1 column/16 pages on load/open/unrelated save. Show `自定义：4列 · 13张` with row choices unselected. First row selection retains columns. First column selection uses rows=min(4,ceil(old_visible/old_columns)). First explicit dimension edit persists a rectangle; never normalize just on open.
- Read-only disables dimension mutation with existing reason. Save failure retains effective dimensions and Unsaved/Retry; durable success requires readback.
- Document overrides stay session-owned. Reconfigure: candidate document -> descriptor -> core acceptance -> document assignment -> cache invalidation -> close. Failure preserves old state. Identical override skips the transaction entirely.

## 9. Agent prompt guide

Page panel: GPUI palette.card, 12px radius, 16px padding, 8px grid gaps, Typography 20/14/12, configured sans, 44px field, palette.border boundary and 2px palette.focus ring. Pointer displacement is immediate 1px; no keyboard animations.

Menu: same tokens, 320px preferred width, >=44px rows, reserved check column, left labels and quiet Auto result. Anchor 8px above trigger while rendering workbench beneath.

## Sequence, rollback and verification limits

Page-browser child implements contextual host first; menu child reuses its API. No dependency/source migration. Revert per child with existing TOML keys remaining compatible. No full-screen fallback for positioning defects.

Task HTML prototype uses illustrative thumbnails and exact tokens, not real raster output. Browser review cannot establish GPUI input, PDFium quality or live audience behavior. Native acceptance follows implementation/build/input tests, with normal-window smoke and relevant themes/locales, no exhaustive size/DPI matrix.

## Normal-window geometry review, 2026-10-18

The first native 1100x760 run exposed duplicate selected checks and a 4x4 grid that showed only about two and a half rows. The checker reproduced a 97px grid overflow after compacting header/density. Coordinator approved adaptive above-toolbar height instead of treating the initial72% target as a hard cap. This is an evidence-based adjustment of the stated starting geometry, without a new UX mode, dimension matrix or scope expansion. Preserve the failed screenshots and bind final acceptance to the corrected build.
