# Design and cross-child contracts

## 1. Direction and architecture

The existing customized `brand:claude` source in root `DESIGN.md` remains the foundation. Extend it during implementation, never catalog-apply over it. Proposed changes below are planning contracts, not claims about a rendered result. Native GPUI uses one Rust token layer, `crates/ferrostage-ui/src/theme.rs`; retain current palette, light/dark/system handling, CJK fallback and home brand. No browser prototype can establish native acceptance.

Child order: settings/design/font foundation, then workbench, then displays. This keeps changes to shared Presenter buttons, theme, locales and shell tests sequential. Parent owns requirements and final integration only. Each child owns its detailed design/implementation document; this file owns shared invariants. All children remain planning until the user approves the final plan and the selected child is activated.

## 2. Visual structure

| Surface | Proposed composition | Hierarchy |
| --- | --- | --- |
| Loaded PDF | Quiet top document/status band, unchanged slide/notes workspace, one session/tool strip | Filename and countdown first; page jump secondary; wall clock/utilities quiet. |
| Session strip | Play icon + Start, Previous, Next; compact page progress; blanking group; mutually exclusive tools; secondary document/grid actions | Start has the clay fill. Stop occupies its place during Presenting and uses semantic danger treatment. No standalone full-width Start footer. |
| Settings | Header with Close; quiet category rail; readable main column with grouped rows; save feedback aligned to content | Section headings and fields are sans. Label/help left, controls right in normal windows. Background belongs to a section or preview, not every field. |
| General | Appearance (language/theme/font/size), Startup, Recent files | Put short explanations beside the setting they qualify. Recent count/limit and content-sized Clear sit with the section heading. |
| Displays | Compact toolbar/count, bounded spatial diagram, role summary, three corresponding assignment rows, local Apply/Cancel | Actual role and pending selection are visually and verbally distinct. Fullscreen preferences form two short ordinary rows. |

Use shared spacing roles based on 4/8/12/16/24/32; control radius 8, group radius 12, status label radius 4. Retain >=44 logical interactive targets, without making every status label a 44px button. Passive labels use quiet text/surface, never command-like outlines. Selected tool has a check/indicator and label, not color alone. Black/white are small labeled swatches inside a regular control, not bright slabs. Destructive Clear remains visually secondary and does not resemble Start.

Normal-window proposal: settings form max width 960 logical px; display category up to 1120 for topology plus rows. Left-align after the rail rather than centering isolated cards. Field rows use a minimum 56px with content-driven height; recent rows approximately 52–64px depending on path. These are implementation targets, not a new geometry qualification matrix. Diagram viewport target 240px high, <=420px wide beside the assignment rows or <=720px when stacked, with a centered contain-fit desktop preserving relative positions and aspect. Put the allocation overview first and fullscreen preferences after it. Application controls do not grow with the diagram. All sizes come from named theme roles when implemented.

## 3. UI typography boundary

Add compatible `general.ui_font_family: String` (default empty string means system default) and `general.ui_font_size: u32` (default 16, accepted 12..=24 logical px, step 1). These are UI base-size settings, not PDF point sizes. Existing role sizes scale by `ui_font_size / 16`; do not replace every hierarchy role with one size. Font family applies to interface headings, labels, filenames, menus, settings and tooltip/error text. The dedicated home wordmark keeps its brand face; timers keep their monospaced role while following the base-size factor. OS titlebar fonts are OS-owned.

The UI lists System default plus locally available font families from the pinned GPUI text system, with a bounded searchable list and a mixed Chinese/Latin preview. Enumerate on first open and cache for the session, never in every render/tick. No font downloads or new font library. Missing requested family retains its saved value, resolves to the current system/CJK fallback, and displays a local warning. Font availability is a UI/platform fact, not a settings parser concern. Selecting default clears the requested override; restore default size is explicit. Apply live via existing preference edits and save flow, without closing settings or moving windows.

Introduce only the shared UI typography resolver needed by these consumers. Audit explicit `.font` / `.text_size` in Presenter, home, settings, modal, display and tool rendering so inheritance is not overridden accidentally. Do not apply it to PDF/raster notes, media/ink geometry, audience pixels or OS controls. Large font choices may increase row height. Do not clip controls to enforce compactness.

Schema version stays 1 for optional compatible leaves. Update defaults, leaf parsing and merge together. Absent/invalid size uses default with leaf diagnostics; invalid UI edit is rejected, not silently defaulted. Preserve unknown TOML keys, future-version read-only policy, unique-sibling atomic save and unsaved Retry. See child settings design for tests and rollback.

## 4. Monitor state layers

| Layer | Authority | Label examples |
| --- | --- | --- |
| Actual | Observed currently owned native window's `window.display(cx)` mapped uniquely to MonitorId | “应用在此”, “PDF 正在此播放” |
| Configured | Existing effective settings and role resolution | “PDF 目标，尚未演示”, “应用配置位置” when different from actual |
| Draft | DisplayDraft for current topology revision | “待应用：应用”, “待应用：PDF” |

Use one Presenter-only projection for diagram, summary and rows. Do not call WindowRecord.monitor/display actual, and do not persist user-drag observations as allocation changes. Per-window observation storage is associated with the native handle and released when that owner retires. Bounds observers sample actual display, initial creation samples once, Shell consumes observations at its existing tick boundary and notifies only when resolved location changes. No new OS monitor polling loop or render-time Shell/window borrowing. Refresh/Apply/opening a panel resamples current owned records. Check native behavior during implementation; if movement does not deliver the existing bounds event, add the narrow native display-change hook rather than guessing.

Actual resolution must respect current mapping validity/revision. Ambiguous or unavailable mapping yields explicit unknown; stale events cannot resurrect a closed/retired window. Window crossing uses GPUI's chosen owning display, not mixed-unit rectangle intersection. Transaction replacement keeps current markers until the new windows are confirmed. No audience window in Idle/Ready means “not presenting”, even when a target is configured. Single-monitor windowed audience may truthfully show both roles on one screen while Presenting. No PDF target says disabled, not unknown.

The schematic shows the same number, orientation and role markers as its row for every detected screen, including “未使用” where applicable. Actual labels are solid/checked with text, pending roles outlined and explicitly prefixed. Selection updates draft markers only; Cancel discards those markers; Apply follows existing transaction and failure paths. Identify uses the same FerroStage numbering, with the existing caveat that these are not Windows numbers. Screen selection may emphasize its existing row; new drag-to-reassign, swap shortcuts or automatic reassignment are out of scope.

## 5. Settings scrolling and feedback

The settings host owns the single content scroll. Remove the fixed 650/450px embed and the embedded panel's second scroll. The standalone Displays dialog reuses the same body with its own single scroll and local action footer. Separate shared body from host framing only as much as those two existing hosts require; no general layout framework.

Global preference status means preferences saved/unsaved/save failed, not draft allocation committed. While allocation is dirty, show “显示器分配待应用” beside its actions. General/fullscreen preference rows keep existing immediate-save behavior and say so locally. Display Cancel discards allocation edits while retaining ordinary already-saved preferences. Close/Escape must never Apply implicitly. Keep an outer settings Close and local allocation Cancel with distinct labels and scopes. Errors and Retry remain visible; technical monitor details collapse individually, but degraded/missing/stale warnings do not disappear with that disclosure.

## 6. Root DESIGN update contract

Settings child updates Typography, settings Layout/Components, general control vocabulary and the QA scope note before code. Workbench child updates document Layout/status/tool/session controls, replacing the old pill and separate Start assumptions. Displays child updates display-assignment, fit/scroll and role-layer contracts. Preserve frontmatter/provenance and existing compatible tokens; do not replace the palette, rewrite unrelated home guidance or claim native verification in DESIGN. Reconcile old exhaustive QA wording with this task's core/normal-window acceptance scope, retaining basic keyboard/focus/contrast constraints.

## 7. Rollout and exclusions

Each child is a reviewable checkpoint in the listed order; final parent review covers cross-child state and home regression. No dependency migration, GPUI/vendor patches, new licensing exception, PDF rewrite, raster/content manipulation, core state-machine change, device identity algorithm change or unrelated optimization. Rollback a child as a coherent code/docs/test diff. Font rollback removes known-key loading/merge/UI additions together; new leaves remain harmless unknown TOML in an older binary. Runtime location observations are session-only and need no migration.
