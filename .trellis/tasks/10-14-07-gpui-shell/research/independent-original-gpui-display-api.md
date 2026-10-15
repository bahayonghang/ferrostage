# GPUI display and input API audit — task07

Status at 2026-10-14: source research completed; production route selection is blocked by the unchanged dependency gate. This document records inspected APIs and the intended integration contract. It is not implementation, a running-window result, or Gate B acceptance. See [dependency-gate.md](dependency-gate.md) for native compile and policy results.

## Exact sources and route choice

The preferred source is Zed tag `v1.23.3`, resolved by the spike lockfile to commit `96a1b1b5389efff473fc2306f7f8c690f2e1a417`. Its GPUI package version is `0.2.2`; this is the permitted tagged source, not a crates.io `gpui 0.2.x` substitution. The platform is now split: startup is `gpui_platform::application()`, which selects `gpui_windows` on Windows. Both exact-tag packages were built with default features disabled in the isolated spike.

The other permitted family was checked at `gpui-kit =0.7.3`, `gpui-component =0.7.3`, and `gpui-pre` / `gpui-pre-platform =0.3.9`. The pre package metadata identifies Zed revision `93d745668805029476e6d7fbb19652116fed51de`. Its Windows display identity algorithm matches the tagged implementation; it also exposes a refresh-interval method. It does not remove the policy blockers, even with kit, base and component omitted. No production route is selected or linked.

Tagged primary files are recoverable from `https://github.com/zed-industries/zed/blob/v1.23.3/` plus these paths:

| Subject | Source path |
| --- | --- |
| Startup/backend | `crates/gpui_platform/src/gpui_platform.rs` |
| Windows identity, DPI, enumeration | `crates/gpui_windows/src/display.rs` |
| Windows window creation/fallback | `crates/gpui_windows/src/window.rs` |
| Public display/window options | `crates/gpui/src/platform.rs` |
| Window methods | `crates/gpui/src/window.rs` |
| Application actions, bindings, file chooser | `crates/gpui/src/app.rs` |
| Keystroke parser | `crates/gpui/src/platform/keystroke.rs` |
| Accessibility/focus examples | `crates/gpui/examples/a11y.rs`, `crates/gpui/examples/tab_stop.rs` |

Local full source is `C:/Users/lyh/.cargo/git/checkouts/zed-a70e2ad075855582/96a1b1b`. Earlier raw-file copies remain under `cache/gpui-v1.23.3`. Registry sources are under `C:/Users/lyh/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f`. `evidence-manifest.json` records the actual read source paths and SHA256 values; the independent route fixtures retain exact manifests, source and lockfiles.

## Display identity and units

The public `PlatformDisplay` interface supplies `id()`, `uuid()`, `bounds()` and `visible_bounds()`. Bounds are logical `Pixels`; physical bounds and monitor scale are private Windows implementation details. In the tagged implementation:

1. `DisplayId` is the transient `HMONITOR` cast to `u64`. It is not a durable settings ID.
2. `WindowsDisplay::new` reads a zero-initialized `MONITORINFOEXW`. The UUID is UUIDv5 in the DNS namespace over **all 32 UTF-16 code units** of `szDevice`, including trailing zeroes, with each code unit encoded as two big-endian bytes. Hashing only the visible string or UTF-8 would be wrong.
3. The backend uses `GetDpiForMonitor(MDT_EFFECTIVE_DPI)` divided by 96. It divides physical origins and extents by this factor for logical bounds. The platform03 `GdiDeviceCapsRatio` diagnostic must not be reinterpreted as this effective scale.
4. `WindowsDisplay::displays()` filters out individual monitor-info/DPI failures; its Win32 enumeration logs an error and can return collected entries. A nonempty GPUI result alone cannot establish complete topology.

The agreed future bridge can use existing `MonitorInfo.connection_name` (the GDI connection name) without adding an HWND/HMONITOR field or introducing unsafe code to FerroStage. Encode the entire name to UTF-16, require it to be nonempty, free of embedded NUL, and short enough for a terminator in the 32-unit array, then zero-fill and apply exactly the upstream algorithm. If the platform representation cannot reproduce the complete name, fail mapping.

Before Start, validate a one-to-one mapping against the complete successful platform03 enumeration: unique `MonitorId`s, connection names, derived UUIDs, GPUI UUIDs and GPUI display IDs; equal counts; exact UUID set equality; every configured role resolved. Reject zero displays, duplicate/ambiguous identities, UUID errors, unmatched/partial enumeration, or a selected role without a mapping. Do not fall back to a screen index, geometry, primary display or DPI estimate. Persist only `MonitorId`/`MonitorRole`; GPUI IDs and UUIDs remain ephemeral adapter values. Pure tests and a real startup enumeration comparison are still required.

## Window creation, migration and safety

`WindowOptions` has a `display_id`, fullscreen bounds, titlebar controls, focus/show controls and minimum size. The Windows constructor silently tries the primary display when the requested ID cannot be instantiated. Therefore the intended transaction must create the audience with a black `Curtain` snapshot, inspect `window.display(cx).id()` after creation, and close it without Start if the actual display differs or cannot be verified. Requested options are not placement evidence.

The audited public window API has `resize` but no arbitrary position setter. Role changes should prepare replacement windows. The root approved this design direction, pending implementation:

- Create and verify all required replacements before retiring any old window. An audience replacement initially reads Curtain.
- Preserve the controller's current page, stage and timer through Swap; do not reset it through Stop/Start.
- On partial creation or verification failure, close only new windows, restore the old snapshot/bindings, and keep the old windows usable.
- Track window identity in close callbacks so retiring an old window cannot stop a replacement session or quit the new presenter. Update presenter focus and modal destinations when handles change.
- Apply resolved roles to the authoritative `display.monitors` settings value. Save failure retains session values and dirty state; no independent swap flag.

Normal lifecycle remains create verified Curtain audience → core Start → Live. A failed create/placement check leaves Ready, no timer start and no leaked audience. Stop means Curtain → core Stop → close. No audience exists in Idle/Ready. Only `Entity<AudienceProjectionSnapshot>` reaches the audience view; no controller, document, notes, path, worker or cache handle is allowed there.

Topology changes must continue to come from the single platform03 poller. Loss of the audience, unavailable topology or zero displays curtains, stops and closes; an invalid presenter moves to an available windowed display. Recovery never auto-starts. These are planned contracts, not tested native behavior.

## Actions, focus and keyboard syntax

The tagged `App::bind_keys` / `KeyBinding` and typed `App::on_action` APIs support application-level routing. `App::on_action` is reached in bubbling if a view did not consume the action. Routing must work while either window has focus. Popup/file/settings intents first call the current presenter's `Window::activate_window`; `App::prompt_for_paths` provides native file selection, and `ExternalPaths` supports file drops. File import continues through the existing single worker's read-only metadata probe, with old state preserved on failure.

`Keystroke::parse` recognizes `ctrl`, `alt`, `shift`, `fn`, `cmd`, `super` and `win` prefixes separated by `-`. Named keys are normalized to lowercase; an uppercase single letter such as `B` means `shift-b`. Existing human-readable settings defaults therefore need a deliberate normalization adapter for intended unshifted letters. Validate the supported key grammar and canonicalize before detecting conflicts; the parser alone accepts more named strings than a useful settings grammar. Disable conflicting bindings and show a warning. Escape first consumes a modal or preview, and only an unconsumed Escape stops presenting.

Focus/accessibility source examples expose `FocusHandle`, tab order/stop controls, next/previous focus, roles and accessible labels/disabled state. Window appearance observation exists for system-theme changes. None of these API findings prove actual keyboard reachability, accessibility announcement or system-theme event handling. Windows `gpu_specs()` currently returns no usable device details; no GPU-runtime claim follows from compiling the spike.

## Resume and verification boundary

Resolve the dependency decision before implementing feature/CI wiring or `ui/theme.rs`. Then implement the root `DESIGN.md` tokens and controls, keeping PDF/media colors and the black audience curtain unchanged; task07 owns the only theme foundation for08/09. Required regressions include ambiguous mapping, lifecycle failures, partial replacement creation, rollback and retired-window callbacks, focus intents, Escape ordering and conflicting keys.

The actual product build, `just ci`, feature clippy, core dependency check and app runtime are **NOT_RUN for task07**. The spike executables have not been launched. No display placement, fullscreen, role swap, start/stop, focus, theme, screenshot, accessibility, import dialog/drop or timing acceptance is claimed. Root alone controls native UI verification after a viable product build. Current hardware has three displays; setting roles to None cannot establish real single/zero-display acceptance. Physical unplug/port changes are explicitly unavailable this turn and remain NOT_RUN. Gate B is not passed.
