# ADR-007: Monitor roles

## Status

Accepted for the core/platform contracts. Native static enumeration is verified; physical port-change/hotplug acceptance and GPUI placement are pending.

## Context

A display index is not stable across three monitors, replug, or restart. A saved audience display must not silently move to another screen. The current source has `MonitorRole` and settings bindings with string ids; legacy `presenter_monitor` / `audience_monitor` indexes are retained as unknown settings with a rebind warning. `MonitorId`, pure topology resolution, owned Windows enumeration, and a cancellable polling watcher are implemented. Task 04 removed the independent `screens_swapped` flag; core rejects the app-only swap intent without mutating state.

## Decision

Displays are `MonitorId(String)` plus `MonitorRole` (`Presenter | Audience | None`). They are not indexes. `MonitorRole` is a core type. Settings serde uses lowercase role names. `MonitorId` lives in the same core module. Session lifecycle does not define display types.

`enumerate_monitors` returns `Result<Vec<MonitorInfo>, PlatformError>`. `MonitorInfo` carries the id, name, native pixel size, physical desktop position, DPI/scale, primary flag, and identity source. Zero monitors is an empty list. An OS failure is `PlatformError`, not an empty list.

`resolve_roles` is pure. It returns `ResolvedTopology`:

- `presenter: Option<MonitorId>`
- `audience: Option<MonitorId>`
- `ignored`
- `source`
- `warnings`

Warning names: `NoMonitors`, `SavedAudienceMissing`, `DuplicateAudience`, `DuplicatePresenter`, `PresenterMissingFallbackPrimary`, `DuplicateBinding`, `AmbiguousMonitorId`.

There is at most one `Audience`.

`MonitorId` is not a display index. Prefer a device path or EDID. Name and resolution are a degraded fallback, and `MonitorInfo` records that identity source and its stability level. The id is not promised to survive a port change.

Resolution rules:

- No saved record: the primary monitor is `Presenter`. If none is primary, the first id in sort order is `Presenter`. With two or more monitors, `Audience` is the largest remaining monitor by pixel area, excluding the resolved Presenter id. Equal areas break ties by `MonitorId` sort.
- One monitor resolves to `Presenter` on that screen and `audience = None`, saved record or not. The same-screen audience window is fullscreen when `display.single_monitor_mode = "fullscreen_audience"` (default) and windowed when it is `"windowed_audience"`. During fullscreen same-screen playback, shortcuts that activate the presenter window do nothing.
- A saved record disables the audience heuristic. A missing saved Audience, or an explicit `None`, does not take another monitor (`SavedAudienceMissing` when the saved Audience is absent).
- A missing Presenter may fall back to the primary after the saved Audience id is excluded, and emits `PresenterMissingFallbackPrimary`. Zero monitors yields `presenter = None` (`NoMonitors`).
- Each `MonitorId` is normalized to one role first. The first duplicate binding wins (`DuplicateBinding`). Duplicate `Audience` or `Presenter` roles then keep the first valid binding in configuration order; the rest become `None` and warn (`DuplicateAudience`, `DuplicatePresenter`).
- If two physical monitors collapse to the same id, do not restore `Audience` automatically (`AmbiguousMonitorId`). A measured identity change asks the speaker to rebind.

The only authority is `display.monitors`. `SwapScreens` is an intent to the app role coordinator: one settings transaction exchanges the two role bindings and moves the windows. Choosing a new Audience in that transaction sets the previous Audience to `None`. There is no `display.swap_roles` key and no independent swap flag. An external file that still contains `display.swap_roles` keeps the key as unknown history and does not execute it. Page and crop do not change.

Hot-plug while `Presenting`, if the Audience disappears: publish `Curtain`, then `Stop`, destroy the audience window, and return to `Ready`. The presenter window stays windowed on a valid Presenter display. A newly attached monitor does not take a role. Reconnect does not call `Start`. Zero monitors wait for recovery, and recovery still requires an explicit `Start`.

An ambiguous position-and-size identity mapping refuses `Start`.

## Consequences

Task 07 implements the binding coordinator and consumes the core/platform topology API. Legacy index fields remain non-authoritative. Native identity is preferred over position and size. This ADR does not relax the workspace `unsafe_code = "forbid"` lint.

## Windows implementation and failure boundary

Task 03 pins Windows-only `winsafe =0.0.29` (MIT, gdi/user APIs). The initially tried display-info dependency failed the advisory gate and was removed; no exception was added. Safe EnumDisplayDevices queries own copied device-interface strings before advancing the wrapper iterator. Active non-mirroring GDI connections must correspond one-to-one with geometry, and a second device query must agree. Partial, duplicate, invalid, or racing results return typed errors. A guarded desktop DC and EnumDisplayMonitors provide per-monitor DCs; GetDeviceCaps physical width must agree with EnumDisplaySettings, whose DEVMODE output is physical pixels. Native handles never become persisted IDs.

Device-interface identity is `ConnectionScoped`, not an EDID or a port-independent guarantee. Missing interface paths use `NameAndResolution`/`Degraded`; collisions disable Audience. DEVMODE geometry is in physical desktop pixels. `ScaleSource::GdiDeviceCapsRatio` explicitly identifies the per-monitor DESKTOPHORZRES/HORZRES ratio in the current calling context, not effective DPI; task 07 must validate the actual GPUI context before placement.

The pure resolver accepts `&[MonitorInfo]` and `&[(MonitorId, MonitorRole)]`; app converts settings bindings. Production platform does not depend on settings. `diff` also reports ambiguous next identities.

`watch_topology(Sender<TopologyEvent>)` uses explicit 200 ms polling and a 200 ms stable-change interval, with no hidden event window. Initial topology is immediate. Errors immediately emit `Unavailable { error, last_known }`; last_known is diagnostic only and must never enable Audience. Recovery emits `Changed` even when topology is unchanged. Stop/Drop cancel the wait and join; in-flight OS calls cannot be interrupted. Non-Windows enumeration/watch returns `UnsupportedPlatform`.

Evidence is in task 03 research/verification.md. Fake-clock event tests and a static three-screen probe do not establish physical unplug latency or identity stability.
