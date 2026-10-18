# Source analysis

Read-only investigation on 2026-10-18, branch dev, initial clean worktree. No product checks or native acceptance were executed during planning. Available commands: python, cargo, just. The referenced global RTK.md does not exist at the supplied path; no RTK-specific workflow was inferred.

## Confirmed source facts

| Evidence | Implication |
| --- | --- |
| `crates/ferrostage-ui/src/audience.rs:9`, `:10`, `:16`, `:71`, `:95` | Audience receives a snapshot entity and emits status events. Its root has no navigation handlers; extend outgoing events rather than incoming data. |
| `crates/ferrostage-ui/src/shell.rs:970`, `:1009`, `:1015`, `:1028` | Audience is created with snapshot-only data. Existing event subscriptions already validate source-window identity; reuse that pattern for input, but permit only committed audience identity. |
| `crates/ferrostage-ui/src/shell.rs:362`, `:369`, `:418`, `:605`, `:609` | Shell owns busy states, projection publication and Intent dispatch. Next maps to NextStep; a specialized audience gate can route into it. |
| `crates/ferrostage-core/src/state.rs:237`, `:314` | NextStep uses slide-map next_step, not page index arithmetic or NextLogicalSlide. End of deck remains unchanged. |
| `crates/ferrostage-ui/src/presenter/workbench_view.rs:268`, `:323`, `:341` | Tool/media mouse input belongs to presenter slide surfaces. Audience click does not need to inherit presenter Tool selection. |
| `crates/ferrostage-settings/src/schema.rs:180`, `:191`; `src/parse/sections.rs:126`; `src/merge.rs:66` | The display section is a suitable existing home for a playback-window bool, but schema/parser/merge must all change. |
| `crates/ferrostage-ui/src/settings_panel.rs:67`; `src/presenter/settings_view.rs:202`, `:493`, `:625` | Field-driven toggles already render in Displays, with label derivation, pointer/keyboard paths and optional helper text. |
| `crates/ferrostage-ui/src/shell/preferences.rs:426`, `:447` | Whole-display equality triggers assign_displays. Normalize the new non-placement leaf in the comparison copy to avoid unnecessary window replacement. |
| `crates/ferrostage-settings/src/parse.rs:201` | Startup writeback does not backfill each missing optional leaf in a complete current document. Do not promise automatic immediate disk insertion merely because the new default is effective. Explicit save merges the leaf. |
| `crates/ferrostage-settings/src/tests/parse.rs:18`; `src/tests/store.rs:59`, `:167`, `:226`; `crates/ferrostage-ui/src/shell/preferences_tests.rs:2` | Existing default-roundtrip, unknown-key, failed-save/retry, future-version and live-consumer fixtures can support meaningful regressions. |
| `crates/ferrostage-ui/src/shell.rs:3301`, `:3334`, `:3461` | Existing headless shell/window/redraw test support can drive actual Audience input without a native display or PDFium DLL. |

## Framework evidence

The pinned vendored implementation is primary evidence for framework API behavior:

- `vendor/gpui/src/elements/div.rs:1584`: stateful `on_click`; root needs an ID.
- `:2951`: records mouse-down when the hitbox is hovered, normally for the left button.
- `:3080`: consumes pending mouse-down on mouse-up capture; outside release clears it without emitting a click.
- `:3104`: constructs a Mouse ClickEvent and invokes the listener during bubble.
- `:2974`: movement threshold applies when a drag listener is installed. A bare on_click is not a custom drag recognizer. Plan intentionally avoids promising full drag cancellation or changing the vendor implementation.
- `vendor/gpui/src/interactive.rs:220`, `:290`: MouseClickEvent exposes down/up; ClickEvent also has keyboard activation. Restrict new input to the mouse form.

## Tradeoffs and approved product intent

- Recommended default true delivers the requested click behavior immediately; false preserves old-user input behavior but requires discovery and setup. The user explicitly approved default true and implementation on 2026-10-18.
- Recommended blanking rule ignores audience clicks during black/white output to prevent hidden page changes. Existing keyboard/button semantics remain intact. The user explicitly approved this blanking rule on 2026-10-18.
- Client-area-wide handling includes black margins and media pixels; adding hyperlink/media hit testing would exceed this feature and expose additional audience inputs.
- Rendering stays snapshot-only. Outgoing input events do not authorize Audience reads of controller/settings/document data.

## Validation scope

Use focused storage tests, real GPUI input tests, default CI and GPUI/PDFium product build/lint. Native smoke must separately validate first-click activation and normal fullscreen/windowed playback. No performance/CPU claims and no exhaustive size/DPI work are required.
