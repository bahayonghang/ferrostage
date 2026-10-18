# Visual and source audit

## Evidence and limits

Reviewed on 2026-10-18 at source `7c25e46d1868f5520b8c2225c2ab89acfe9cfbed`, branch `codex/home-design-refactor`; working tree was clean before planning. Screenshots are user-provided evidence, copied unchanged into `research/screenshots/`: 01 workbench, 02 General, 03 display topology, 04 assignment rows. They are not new native captures. Runtime dimensions, scale and exact executable identity of those attachments are unknown. No product code, live application, or build was changed/run during this planning turn.

Read `ui` screenshot iteration and full design-reference rules, `design-md` existing-source/customization/native implementation/brand rules, and Trellis start/brainstorm guidance. Existing root `DESIGN.md`, not a fresh catalog entry, is authoritative: customized `brand:claude`, warm utility surfaces, clay primary action, native GPUI. Do not reapply the catalog or replace the shipped home direction. `C:/Users/lyh/.codex/RTK.md`, referenced by AGENTS, was absent; no substitute instructions were invented. Prior home planning artifacts already preserve the explicit user direction to stop exhaustive size/DPI/non-core acceptance, so additional conversation-history mining was unnecessary.

## Diagnosis

| ID / priority | Screenshot observation | Source evidence | Consequence / repair |
| --- | --- | --- | --- |
| F1 / P1 | Filename isolated far left; status, long progress, timer, clock and four outlined utilities crowd the right | `crates/ferrostage-ui/src/presenter.rs:377`, `:405`, `:419`, `:429`, `:435` | Identity consumes free space while compact utilities compete. Group filename with quiet session status; put concise actionable page progress near navigation; reduce utilities' permanent borders. |
| F2 / P1 | All lower controls read as separate badges, with square navigation, swatches, pills and destructive actions of similar weight | `crates/ferrostage-ui/src/presenter/workbench_view.rs:395`, `:412`, `:428`; `DESIGN.md` Components explicitly describes pills | This is a design-contract change, not only a spacing tweak. Replace the mixed face vocabulary with grouped action/tool controls, keep selection and blank-state semantics explicit. |
| F3 / P1 | Start spans a second entire row below navigation | `crates/ferrostage-ui/src/presenter.rs:879`, `:882`, `:885`, `:889` | Tools and Start are independently composed. One session-control strip must own Start/Stop, Previous/Next and tools, reclaiming the redundant footer. |
| F4 / P1 | General's short fields occupy very wide, tall background blocks | `crates/ferrostage-ui/src/presenter/settings_view.rs:48`, `:63`, `:357`, `:441` | Generic rows always stack label above controls and fill remaining width. Use bounded readable content, grouped rows, label/help on left and controls on right. |
| F5 / P2 | Categories are seven equal outlined buttons; heading uses serif while field labels use sans | `crates/ferrostage-ui/src/presenter/settings_view.rs:29`, `:36`, `:339` | Navigation looks like an action collection. Use a quiet rail, text weight and selected surface; settings headings use UI typography, reserve serif for the home brand. |
| F6 / P1 | Clear recents is a full-width command; remaining files are raw path paragraphs | `crates/ferrostage-ui/src/presenter/settings_view.rs:78`, `:86`; basename/parent pattern already at `presenter/home_view.rs:94` | Put count/limit/clear beside a Recent files heading; filename plus parent context in compact informational rows. No new open/pin/remove actions. |
| F7 / P1 | No family/size option; current font rendering is fixed | `crates/ferrostage-ui/src/theme.rs:16`, `:26`, `:34`; `settings_panel.rs:77`; `ferrostage-settings/src/parse/sections.rs:17`, `merge.rs:48` | Add compatible general keys, live UI typography resolution and persistent edit flow. Existing notes_font_scale is document-preview scaling, not UI typography. |
| F8 / P1 | Huge topology spills below the viewport; role rows require a second scroll position | `crates/ferrostage-ui/src/presenter/settings_view.rs:97`; `display_settings.rs:334`, `:392`, `:625` | Fixed 650px embed plus inner/outer scrolling plus full-width aspect-derived diagram. Remove fixed embed, give the settings host one content scroll, contain diagram in a bounded preview. |
| F9 / P1 | Screen boxes show only numbers; current application/PDF role is absent from the diagram | `crates/ferrostage-ui/src/display_settings.rs:400`, `:414`, `:461`, `:471` | Diagram and rows need the same role projection and numbering. Selected buttons currently describe the draft, not actual window placement. |
| F10 / P1 | Saved/automatically effective footer coexists with display Apply/Cancel | `crates/ferrostage-ui/src/presenter/settings_view.rs:305`; `display_settings.rs:599`, `:640`; `shell/preferences.rs:42`, `:59` | Autosaved preferences and transactional display allocation must have separate, truthful feedback. Show an explicit pending-allocation state until Apply; Cancel restores draft only. |
| F11 / P2 | Repeated device-interface/GDI diagnostics dominate each display row | `crates/ferrostage-ui/src/display_settings.rs:550`, `:570` | Default to number, friendly name, resolution/orientation, roles and assignment controls. Technical details become disclosure content; active warnings remain visible. |

## Actual monitor location is not a styling-only change

`DisplayPanel` at `display_settings.rs:296` has monitors, effective settings and a draft, but no actual window-location observations. `Shell::display_panel` at `shell.rs:1413` clones settings and draft. `WindowRecord` at `shell.rs:61` stores requested monitor/display; those fields must not be relabeled as actual after an OS drag. `shell.rs:1123` already reads `window.display(cx)` for logging, demonstrating the native API in the pinned source. Window-bounds observers at `shell.rs:944` and `:960` currently mark observation/logging only. `vendor/gpui/src/window.rs:2992` exposes the display API. `Shell::tick` at `shell.rs:1701` provides an existing coordination boundary.

Keep geometry observation in Shell/window infrastructure. Publish a small Presenter-only location projection; never add theme/settings/controller access or location controls to `AudienceProjectionSnapshot`. Before Start there is no audience window: show the configured PDF destination and explicitly state “not presenting”. Never imply that an already visible preview is an audience window.

## Typography feasibility

`vendor/gpui/src/text_system.rs:284` supplies sorted, deduplicated available font names. `theme.rs` is the existing token layer. `settings_panel.rs:293` parses proposed edits via the canonical TOML loader and rejects warnings for the edited key, retaining unrelated monitor binding data. `shell/preferences.rs:243` applies changes through the existing preflight/save flow. Schema additions require updates to schema/defaults, leaf parser, known-key merge and configuration documentation; Serialize alone does not implement loading or persistence.

## Locked direction and scope

- Visual thesis: warm, quiet presenter utility, with a continuous neutral workspace and one clay session action. No decorative cards, shadows, gradients, platform-style migration or downloaded fonts.
- Content plan: orient with filename and session state; enable presentation/navigation; keep slides and notes dominant. Settings use category, section, compact rows, local explanation, truthful save feedback.
- Interaction thesis: no ornamental animation. Preserve immediate shared pressed-state feedback and visible focus; repeated page navigation and keyboard activation do not animate.
- Strategy: native GPUI style methods backed by the single `theme.rs` layer, no CSS/web surface. Existing screen/workbench data ownership remains intact.
- Preserve: home composition, PDF pixels/aspect/letterboxing, notes priority and 55–80% range, direct manual page jumps, timers/media/ink semantics, MonitorId identity and existing Apply/Cancel transactions.
- User clarified font scope: UI family plus size only. No PDF or rasterized-notes font substitution.
- Verification budget: core focused regressions, normal-window native before/after visuals and existing build/lint gates. No new minimum-window/breakpoint/DPI/performance matrix. Do not call unrun native work verified.

## Why these choices

An icon-only Start would save width but hide the most consequential action; use a larger play icon plus explicit Start label, replaced in place by Stop. A new brand preset would create work without addressing allocation semantics; retain the existing brand source. A smaller topology alone would still lie about pending versus actual location; add the data contract. Global reduction of control targets would hide the density problem; remove wasted containers and repeated descriptions instead.
