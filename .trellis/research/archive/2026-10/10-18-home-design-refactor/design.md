# Proposed technical and visual design

## Contract and direction

Target: presenter Idle home, including app-owned header/body/footer and native title string. Preserve: Ready/Presenting layout, shared input/accessibility, modal behavior, error/save feedback, imports, and audience isolation.

- **Visual thesis:** warm editorial utility, restrained clay action, flat warm surfaces, and one distinctive FerroStage wordmark.
- **Content plan:** brand and quiet utilities, brief instruction and one Open action, then recent files; conditional feedback stays visible near these controls.
- **Interaction thesis:** immediate hover/pressed/focus feedback only. Preserve the existing 1px pointer press offset without animation. No entrance motion or keyboard animation.

Context: desktop presenter preparing a PDF. Signature: dedicated brand typography. Constraints: Rust/GPUI, maintained source, both shipped locales/themes, 44px targets and actual 320x240 minimum. Styling strategy: the single `theme.rs` GPUI token layer, no CSS/web stack.

## Design source and color

Root `DESIGN.md` is authoritative and records `brand:claude` provenance. Do not run catalog `apply --force`, install a theme, or replace the document during planning. During approved implementation update only home typography/layout/component/responsive rules, retain provenance and add a customization note.

Retain primary #A34F35, hover #93452F, pressed #823C29, button ink #FFFFFF; light canvas #FAF9F5 and dark #181715 with their current text/focus palettes. Accent is for the main action, not large decorative fields. Recents have no default enclosing border; focus still uses the tested focus role. Decorative separators must not replace interactive focus indicators. No global palette/border changes are needed.

## Typography and complete copy disposition

Sizes are logical pixels at default text scale, proposed design values rather than screenshot-derived DPI measurements.

| Role / current item | Styling or disposition |
| --- | --- |
| FerroStage wordmark | Dedicated `home_brand` style, Georgia 32/38, weight 600, existing ink, left aligned. Separate from generic display headings. Normal tracking; no gradient, faux logo or copied identity. |
| Duplicate header filename fallback | Omit on Idle; preserve real filenames on Ready/Presenting. |
| Native window title | Idle `FerroStage`; preserve informative document/stage title outside Idle. |
| No document stage | Omit on Idle; preserve meaningful document-stage labels. |
| Welcome sentence | One sans instruction: `选择 PDF，准备演示` / `Choose a PDF to present`, 14/22, muted/body. No extra marketing headline. |
| Open PDF | `打开 PDF` / `Open PDF`, sans 14/20 weight 500. Loading uses existing localized progress copy with stable geometry. |
| Recent section | `最近文件` / `Recent files`, sans 12/18 regular, muted; not another page heading. |
| Recent filename | Sans 14/20 weight 500, ink; preserve extension. |
| Recent parent path | Sans 12/18 regular, muted; actual parent context, no invented metadata. |
| Empty recents | `暂无最近文件` / `No recent files`, 14/22 muted; no empty giant list/card. |
| Settings / Displays | Existing explicit localized labels, sans 14/20, quiet utility styling. Keep both destinations; no icon-only ambiguity. |
| Timer and clock | No Idle render. Outside Idle retain current monospace timer/clock, warning labels and `show_clock`; no format/state change in this task. |
| Disabled Start and reason | Omit the complete footer only on Idle; preserve Start/Stop and disabled reasons on document screens. |
| Import/topology/save feedback | Sans 14/22, existing semantic colors and labels; wrap in bounded column; retain alert/Retry behavior. Never hide actionable feedback as cleanup. |

Use `theme::sans_font()` (Segoe UI, Microsoft YaHei UI fallback) for all non-brand home text. No Latin tracking on Chinese; no Georgia/SimSun for routine interface copy. Do not download fonts. Long filenames/paths may truncate as content with full information on focus/hover and in accessibility naming; commands, status, and time metrics must not be ellipsized to force fit. Narrow widths prioritize filename/extension access and full-path discovery over path density.

## Composition and geometry

```text
native title: FerroStage

        FerroStage                   设置   显示器设置

        选择 PDF，准备演示
        [打开 PDF]

        最近文件
        perf-100.pdf
        parent path
        raster.pdf
        parent path

        conditional error / warning / unsaved state and Retry
```

Path labels above illustrate structure only; production uses actual saved paths. This is an information wireframe, not accepted GPUI rendering.

- One centered column: `width = min(760, client_width - 2 * gutter)`. Gutter 32 at >=800px, 16 below. Brand, copy, action, recents and feedback share a left edge, within 2px in geometry checks.
- Top inset 48 at heights >=600, 24 below 600. Top anchored, not vertically centered; recents/errors must not make brand position jump.
- Brand and utilities share a row when they fit. Below 600px width utilities wrap into their own left-aligned row. Use natural widths rather than shrinking copy to squeeze labels. Body scrolls vertically at short heights.
- Spacing uses 8/16/24/32. Instruction follows brand area by 24; Open follows instruction by 16; recents follow Open by 32. Recent row gaps are 0 to 8, not large separated blocks.
- Open face keeps existing 44px minimum height and 16px horizontal padding, natural width with 120px minimum. Shipped labels should remain <=200px wide at default scale. Existing focus reservation makes its outer target about 52px; distinguish face, target and ring.
- Fix width, alignment, grouping and duplication first. Do not reduce global `CONTROL`. Home utilities initially retain current target/focus geometry with quieter fills. A second density system is unnecessary unless native comparison proves otherwise.
- Recent row target 48 to 56px including focus reservation at default scale, two left-aligned text lines, ring within bounds. Rows may grow at increased text scale to avoid clipping.
- Action face radius stays 8. Recent rows are flat with subtle hover/press surfaces, not separate rounded cards. No ornamental shadows, gradients, added marks or animation.

## Source boundaries and reuse

| File | Responsibility |
| --- | --- |
| `DESIGN.md` | Record reviewed home contract before product changes; retain neighboring-screen contracts. |
| `crates/ferrostage-ui/src/theme.rs` | Only necessary home role/spacing tokens; preserve shared button sizes, palettes and contrast tests. |
| `crates/ferrostage-ui/src/presenter.rs` | Idle routing/title, redundant chrome/footer omission, feedback and focus preservation. |
| `crates/ferrostage-ui/src/presenter/home_view.rs` (new if useful) | Home composition/recent rows as Presenter methods over existing captured data and intents; no new app model. |
| `assets/locales/{zh-CN,en-US}/app.toml` | Short instruction, empty history and accessible path copy; preserve key/interpolation parity. |
| `crates/ferrostage-ui/src/shell.rs` tests | Extend actual GPUI fixtures around home geometry/input and workbench preservation. Runtime Shell changes should not be needed. |

Avoid rebuilding the generic button framework. A recent row needs richer content/a different face but the same stable identity, enabled guard, keyboard and accessible dispatch. Prefer a small activation wrapper extracted from `Presenter::button` only if it prevents copying those behaviors. Existing `button` calls keep their API/geometry. No general component configuration abstraction or blanket restyle.

Retain `welcome-open`, `recent-{index}`, `settings`, `displays`, `retry-save`; add `home-content` / `home-brand` for measurements. Idle no longer renders toolbar `open`, `document-title`, `stage-label`, `presenter-timer` or `start`; their document-stage IDs remain. Audit callers/tests assuming they always exist. Modal focus recovery must tolerate removed controls through the existing fallback.

## Data and behavior

Use `PresenterData` prepared from Shell; never synchronously read a leased Shell during render. Derive basename/parent from existing recent strings without filesystem I/O or a second history model. Keep `.take(3)`, order and original import path. No PDF metadata preflight on the UI thread.

Open keeps `Intent::Open` and `can_open`; recents keep `Intent::Import` and existing busy/import authority. Shell still rejects invalid commands even when controls are hidden. Preserve common errors/warnings/dirty/Retry, chooser cancellation and drop handling. Avoid an early return that bypasses feedback, focus or accessibility wiring.

No settings key/migration, timer mutation or audience input. Visibility depends only on Idle. Leaving Idle restores document controls; Home retains existing core cleanup/recents behavior. Keep document-stage timer data production intact; hiding Idle values is not a claimed CPU optimization.

## Compatibility, risk and rollback

- High-risk seams: shared activation, modal return focus, common error/save feedback, unconditional footer. Test actual events and geometry, not helper constants alone.
- Root DESIGN currently allows Georgia for general Idle headings and describes stage visibility. Revise home-specific rules to this contract without weakening 44px/contrast or changing settings/workbench rules.
- Native rendering must establish Latin/CJK matching and wordmark weight. The supplied bitmap has no reliable logical-client/DPI metadata.
- Roll back view/token/copy/design changes together if shared input/workbench regresses. No persisted migration/dependency needs reversal.
- Status: proposed. Root DESIGN/product code remain unchanged in planning. Native screenshots, keyboard interaction, contrast reruns and aesthetic acceptance remain implementation gates.

## Requirement traceability

| Requirement | Mechanism | Acceptance / evidence |
| --- | --- | --- |
| R1 | Dedicated home brand role, stage-aware native title and single identity | AC1, source/tree assertions and native brand/title screenshot |
| R2 | Complete copy/typography disposition table and locale parity | AC2, AC4, AC7, locale checks and native CJK/Latin comparison |
| R3 | Idle-only chrome/footer/timer omission, no state mutation | AC2, AC9, Idle absence and Ready/Presenting presence/behavior checks |
| R4 | Natural primary width, bounded column, rich recent rows over original import paths | AC3-5, measured geometry and actual import/error tests |
| R5 | Explicit column/gutters/wrap/scroll rules and unchanged minimum targets | AC3-4, AC6-7, minimum-size/focus/contrast matrix |
| R6 | Preserve common feedback and Shell authority, reuse stable activation/focus | AC5-6, AC8-9, chooser/drop/busy/modal/save/audience paths |
| R7 | Single token layer, isolated home branch, shared-consumer review | AC9-10, existing workbench regressions, strict gates and native evidence |

## User scope correction, 2026-10-18

The user explicitly directed: do not validate dimensions or similar non-core behavior, and persist this preference globally. Stop further native size/breakpoint/minimum-window/DPI matrices. Their unexecuted cases are no longer blockers for this task. Existing passing regression evidence is retained without more matrix expansion. Continue only core behavior, normal-window visual smoke and necessary build/lint/relevant regressions. No exhaustive accessibility or physical-display qualification is required for this home polish task. The unrelated existing all-features dependency-policy failure remains truthfully reported separately; do not add waivers or expand this UI task into dependency migration.
