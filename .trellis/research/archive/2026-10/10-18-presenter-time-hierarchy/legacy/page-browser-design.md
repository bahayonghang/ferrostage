# Dual-mode page browser, proposed design

Status: approved for implementation on 2026-10-18. D1 requires local notes rejection with no redirect. The wireframes below are design illustrations, not rendered implementation evidence.

## Visual contract

Reuse the existing warm, opaque, upward contextual panel and its 880px preferred width, 16px padding, 12px radius, 8px grid gaps, shared controls and keyboard focus. Keep slide/notes pixels unchanged behind the panel. Add a quiet mutually exclusive choice row below the title/status and above the numeric editor, with no new floating window or global theme.

Example for a document with 95 content pages and 190 physical pages:

```text
跳转页面                                           关闭
当前：内容页 1 / 95 · PDF 1 / 190

页码方式   [✓ 内容页] [PDF 原页码]
内容页码   [ 2        ] / 95       [跳转]
内容页 2 → PDF 3 · 不含已识别的备注页

列 [1][2][3][✓4] × 行 [1][2][3][✓4]
[第1页 · PDF1] [第2页 · PDF3] [第3页 · PDF5] ...
前一组                内容页 1-16 / 95           后一组
```

```text
页码方式   [内容页] [✓ PDF 原页码]
PDF 页码   [ 3        ] / 190      [跳转]
PDF 3 → 内容页 2

[PDF1 · 内容1] [PDF2 · 备注] [PDF3 · 内容2] ...
前一组                PDF 页 1-16 / 190          后一组
```

These are illustrative numbers, not parity-based mapping. Notes slots use a quiet textual placeholder and explicit non-projectable state, not a rendered notes preview. Avoid shrinking actual slide thumbnails to preserve the old grid height: include the added selector row in the existing panel chrome-height calculation and let only the image/grid region adapt. Do not add new window-size branches.

Use existing selected button surface/checkmark and visible focus, not color alone. Mode controls remain available with read-only settings. They are local browsing state; row/column settings retain their existing persistence restrictions. Keep input/selection/caret behavior and keyboard submission native.

## Numbering contract

| Mode | Range and mapping | Grid |
| --- | --- | --- |
| Content, default | 1..K, where K is `slide_map.steps().len()`; n maps to `steps[n-1]` | All navigable content pages, consecutive visible numbers |
| PDF original | 1..P, where P is the PDF physical page count; p maps to `PhysicalPage(p-1)` | Original positions including marked notes placeholders |

K is not P/2. It is also not necessarily `groups().len()`: preserve individual overlay steps rather than silently collapsing their jump targets. PDF labels can be repeated, nonnumeric or absent; do not parse them as the numeric editor's identity. The current content sequence remains the source of selectable audience pages. The existing logical-slide/step summary may remain as secondary detail with unambiguous wording.

D1, approved: a recognized notes physical position is not in the content map. Show 备注页不可投影 (and its English counterpart), keep Jump disabled, and do not navigate or redirect to content. The same applies to orphan notes. Notes tiles remain non-projectable placeholders.

## State and event flow

1. Maintain a typed `PageNumberMode` independent of crop/default/layout/overlay-grouping options. The accepted document session owns the remembered mode, initially Content. A new document resets it; close/reopen and same-file reload preserve it. Retire the open browser on a generation-changing reload as today, then reopen against the current map.
2. Capture an ordered browser-entry projection containing number, physical page and selectable-content/notes kind, plus localized labels derived at the view boundary. One owner supplies all consumers; the view must not independently reconstruct ordering/counts.
3. Resolve numeric edits against the selected mode and current generation. Editing/revealing is not committing. Keep raw editor text distinct from the resolved physical target and committed current page.
4. On a mode change, translate a valid target's physical page to the corresponding new number; otherwise use the current content page. Reveal its group and select the resulting input for replacement. Do not reinterpret the old digits unchanged, and do not navigate.
5. Guard mode changes, edits and tile submits with the current browser identity/generation plus the active numbering interpretation. Renew the browser interaction identity on mode changes, or use an equivalent revision check, so already queued old-mode events cannot jump to a different page after a switch. Preserve/refocus the existing editor through the shared focus lifecycle.
6. Submission still reaches `PresentationCommand::JumpTo(PhysicalPage)` only after content-map validation and the approved D1 notes-rejection policy. Closing/outside/Escape retains existing no-navigation and focus-return behavior.

No settings schema, PDF write, core map mutation or `ReconfigureDocument` is needed for numbering changes. Do not route mode changes through the current layout reconfiguration path because it may clear annotations and retire the document.

## Scheduling contract and affected files

`presenter/page_browser.rs` currently derives captions from `map.steps()`, reads `wb.steps` for tiles and `self.data.pages` for the input denominator. `shell/page_browser.rs` resolves edits as physical numbers and reveals within content steps. `Shell::GridMove` also counts content steps. All must use the same active projection after this change. Update the number editor's accessible label with its active mode as well as the visible label and range.

`workbench.rs::desired` currently schedules `Surface::Thumbnail(index)` by content-step index. A visual-only mode selector would leave physical-grid indexes pointing at wrong frames. Supply the active mode-aware entry list to scheduling through a pure model boundary compatible with the default/no-GPUI build. Avoid a dependency from pure Workbench onto a GPUI-only presenter module; put shared projection data in an appropriate existing pure module or explicit input type after checking module gates.

Use at most the configured 16 visible entry slots, including notes placeholders; issue thumbnail requests only for the selectable content entries in that window. Keep surface identity paired with the actual physical page, generation and crop. On mode changes, retire prior thumbnail frames/requests and reject old results that no longer match current wanted keys; placeholder slots never consume note raster data. Keep all thumbnail work absent when the browser is closed. Do not change audience/current/next/notes rendering queues.

Likely affected files: `page_browser.rs`, `presenter/page_browser.rs`, `presenter/number_input.rs`, `shell/page_browser.rs`, the relevant `actions.rs` intent/`shell.rs` dispatch paths, Workbench scheduling and associated tests, paired locales, `DESIGN.md`, and directly contradicted frontend specs. Do not expand settings or core unless actual implementation evidence shows a necessary boundary adjustment.

## Verification

Trace AC12-AC16 in `prd.md` and the integrated plan in `implement.md`. Test irregular mapping, plain PDFs, overlays, all input ranges and D1; actual keyboard/mouse mode changes and no-commit interactions; stale identity/generation events; reopens/reloads; read-only settings; ink/timer preservation; and exact thumbnail ownership/retirement. Native visual evidence covers the normal-window panel in both modes with readable numbers and focused input. Implementation and first normal-window smoke are recorded in verification.md; the wireframes remain design illustrations rather than screenshot evidence.
