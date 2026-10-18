# Time-display research and source findings

Research date: 2026-10-18. These are official documentation references and current repository findings. Product documentation/search excerpts were inspected; external product UIs were not run and their visual geometry was not measured. Proposed pixel values are FerroStage design choices, not values copied from those products.

## Screenshot diagnosis and repair contract

Input: user-provided `codex-clipboard-07cfc8f7-64a2-4a80-93bc-7b92ad2dfc7d.png` in the local temporary folder. Ready workbench, dark palette, presentation timing `00:00`, wall clock `11:17:50`, right-side notes and next-slide preview. The orange outline is the user's annotation, not an app border to reproduce.

Problem: two unlabeled changing readouts share the header's action area. Timer size differs, but spatial grouping still makes the time functions unnecessarily similar and pushes a glance toward utility actions. This is an information-hierarchy problem rather than a broken timer.

Target: the loaded presenter header's two readouts and their placement, including the existing Presenting-only timing controls.

Preserve: slide/notes/next geometry and pixels, header action order/identities, filename access, core timing behavior, settings and audience separation.

Evidence for implementation: same normal native window before/after, Ready and Presenting, focused state observations, rendered bounds and existing behavioral regressions. This planning turn has no after render and claims no visual acceptance.

## Official online references

### PowerPoint Presenter View

Source: [Microsoft Support, Use Presenter View in PowerPoint](https://support.microsoft.com/en-gb/office/what-is-presenter-view-98f31265-9630-41a7-a3f1-9b4736928ee3).

Documented facts: Presenter View includes the slide, next slide and notes; timer pause/reset are identified in the upper-left region; current time is separately described as a pacing aid.

Decision: treat elapsed time and wall clock as distinct semantic roles, and preserve timer controls beside their readout. Do not copy PowerPoint's timer placement because the user specifically asked for the current time at top left. This is a functional reference, not evidence for exact typography or colors.

### Keynote presenter display

Sources: [Apple Support, Rehearse a Keynote presentation on your Mac](https://support.apple.com/en-sa/guide/keynote/tan1cb6ca7a3/mac), [Apple Education, Keynote for Mac: Present](https://education.apple.com/learning-center/T024610A-en_EMEIA).

Documented facts: presenter display configuration includes separate clock and timer elements alongside slide/notes content. Apple documents proportional resizing of clock, timer and labels.

Decision: keep time displays as secondary presenter instruments around the content. Use explicit labels and controlled scale rather than introducing an alarm dashboard. Do not add configurable layout or timer features from Keynote.

### Stable numeric typography

Source: [Apple Developer, monospacedDigitSystemFont(ofSize:weight:)](https://developer.apple.com/documentation/uikit/uifont/monospaceddigitsystemfont%28ofsize%3Aweight%3A%29).

Documented fact: the API provides a fixed-width numeric font option. It is an Apple platform API, not a dependency suitable for this GPUI Windows app.

Decision: preserve the existing Consolas numeric role and reserve value space instead of importing a brand font. Stable digit advances are useful; longer minute counts still need an explicit allocation policy.

## Verified repository findings

| Evidence | Finding and implication |
| --- | --- |
| `presenter.rs:596-637` | Identity already has a shrinkable filename and nonshrinking stage. Place clock before this group without breaking full filename access. |
| `presenter.rs:638-711` | Timer currently follows identity and precedes actions; a separate existing branch handles width below 800 logical px. Preserve that branch rather than redesigning breakpoints. |
| `presenter/workbench_view.rs:30-39,130-153` | Existing timer calculations own count-up/countdown, negative overtime, local HH:MM:SS and warning states. Retain these semantics. |
| `presenter/workbench_view.rs:375-438` | Both numeric strings are rendered in `timer_panel`; timer is mono 24px and clock mono 14px. Pause/reset appear only while Presenting. Split rendering responsibility, not timing ownership. |
| `theme.rs:14-16,23-50,166-195` | Segoe UI/Consolas roles, existing size scaling and complete light/dark palettes already exist. Reuse them. |
| `locale.rs:4-7,48-56` | Catalogs are `assets/locales/{zh-CN,en-US}/app.toml`, with key/interpolation parity checks. |
| `shell/preferences_tests.rs:630` | An existing regression covers settings propagation into workbench timer and clock consumers. Extend only where the visual split changes the contract. |
| `.trellis/spec/frontend/component-guidelines.md:35-76` | Mono timing role, typography scaling, shared activation and normal-window verification are established requirements; no exhaustive size/DPI matrix is implied. |

## Alternative assessment

A, recommended: unboxed 28px timing digits with a quiet inline semantic label; 14px wall clock moves left. Stronger glance hierarchy with little extra visual mass.

B: same timing group inside a low-contrast soft inset. Easier to find, but visually heavier and may look interactive. Reject as the default unless the user prefers this tradeoff.

Large accent badges, saturated backgrounds, progress rings, pulsing separators and removing seconds are outside scope or work against the requested restraint. Timer font replacement is unnecessary because current code already uses an appropriate numeric face.

## Planning result

One bounded task is enough: the presenter refinements share one surface and should receive an integrated review. No independent parent/child deliverables are needed. Requirements are captured, the recommended appearance/defaults are explicit, and the user has approved the combined plan and implementation is underway. No placeholders or generated assets are needed.

## Scope extension: redundant tool and directional notes controls

Second screenshot: the user marks the Navigate toolbar entry and Notes left/right/top/bottom, asks for their removal, and states that notes should default to independent pages. This adds product behavior beyond time-display styling. These are current source findings, not new web claims.

- `workbench.rs:12-19`: `Tool::Navigate` is the default neutral state. `presenter/workbench_view.rs:600-621` renders its standalone toolbar entry. Remove the entry, not the neutral state.
- `shell.rs:469-477`: clicking an already active tool returns to Navigate, ends the stroke, and clears the laser. A replacement control or new Escape behavior is unnecessary. Preserve and test this path for the remaining tools.
- `presenter/layout_navigation.rs:4-11`: six layout choices include four directional crop overrides. The menu has eight total rows after adding independent-notes and physical-navigation toggles.
- `presenter.rs:112-114` and `presenter/layout_navigation.rs:120-227`: eight focus handles/anchors, hard-coded keyboard modulo/indexes and radio/checkbox boundaries all depend on that row count. Removing rendering alone would leave hidden keyboard targets or wrong actions.
- `presenter/layout_navigation.rs:165-169`: the current detected-layout description searches the visible `LAYOUTS` list and unwraps. A shorter list must not remove labels needed to describe an explicit automatic Right/Bottom result.
- `ferrostage-document/src/layout.rs:23-30`: defaults currently combine `layout=None` (Automatic), independent-note recognition on, and physical navigation off. Independent notes already being enabled does not prevent aspect-ratio cropping.
- `ferrostage-document/src/builder.rs:42-57,100-128,138-154`: new load uses default overrides; reload reuses them; Automatic can select Right/Bottom by aspect ratio; independent-note mapping relies on `notes:` labels and excludes those pages from content steps. Do not substitute odd/even pairing.
- `ferrostage-document/src/builder/tests.rs`, test `layout_candidates_overrides_counterexamples_and_reload_scope`: currently expects automatic cropping from default load. Separate explicit Automatic tests from new safe-default tests rather than deleting detector coverage.

Recommendation for review: new documents use explicit No split with independent-note recognition enabled. Keep Automatic and No split as user-selectable choices, because the user marked only the four directional rows for removal. This prevents unwanted default cropping while preserving deliberate opt-in Automatic and lower-layer compatibility. The latest combined design keeps independent-note recognition in this menu and puts numbering choices in the browser, without exposing the old overlay-grouping checkbox as a second physical-page control.

The second request asked for combined implementation. The revised default and consolidated plan require the Trellis final-summary review before activation; planning revisions have not changed product code. The subsequent numbering request below supersedes the earlier recommendation to keep a physical-navigation checkbox in this menu.

## Scope extension: two numbering modes in the jump panel

The third screenshot shows a physical editor range of 190 and content tiles using PDF positions 1, 3, 5... with a content count of 95. The user requests a top-of-panel selector for default content-only numbering and full-PDF numbering. Treat the half-sized total as an example of this deck, not a general page-identification algorithm.

Current source findings:

- `page_browser.rs::PageBrowser` contains edit text, resolved physical target, generation and per-open identity, but no numbering mode.
- `ferrostage-ui/src/lib.rs:21-22,35` gates `page_browser` behind GPUI while Workbench is unconditional. A shared scheduler projection must live on a pure-model boundary, not import that gated module into the default build.
- `shell/page_browser.rs::{open_browser,edit_browser,submit_browser}` initialize/parse one-based physical positions and validate membership in the content map. Recognized notes numbers are currently rejected with `pb-notes-page`.
- `presenter/page_browser.rs::BrowserData::from_shell` derives captions only from `map.steps()`. `page_browser_panel` independently reads those steps, the physical page count and content range/footer. All consumers must change coherently.
- `shell.rs::Intent::GridMove` counts content steps; `workbench.rs::desired` allocates thumbnail indexes by content-step index. A label-only or input-only mode switch would mismatch counts, tiles and actual raster frames.
- `ferrostage-core/src/slide_map.rs` distinguishes all navigable content steps from logical slide groups. Preserve content-page steps for default numbering; do not assume every logical slide has only one physical content page.
- `ferrostage-document/src/builder.rs` uses `physical_pages` to control grouping of repeated-label overlays. It does not mean include notes pages or change browser input numbering. Reusing it would change document semantics, potentially reconfigure/clear ink, and still fail to implement the requested two numeric ranges.
- `assets/locales/zh-CN/app.toml:304-317` currently hard-code physical input terminology, content footer terminology and notes rejection. Both catalogs need explicit per-mode names/range helpers.

Updated design: top-of-panel Content / PDF original selector, coherent mode-aware entries/ranges/targets, default Content per new document, and no document reconfiguration on numbering changes. The old physical-navigation checkbox is removed from Layout & navigation to avoid offering a second control with misleadingly similar wording; its internal overlay-grouping capability is retained. That menu now has three rows. See `page-browser-design.md` for wireframes and implementation contracts.

D1 resolved by the user on 2026-10-18: original-PDF input aimed at a recognized notes page must show 备注页不可投影 and never jump or redirect. The user explicitly approved the rest of the combined plan and requested implementation.
