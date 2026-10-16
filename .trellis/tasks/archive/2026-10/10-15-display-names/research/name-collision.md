# Why three monitors share one name

Date: 2026-10-15. Read-only inspection of the running machine plus the saved enumeration log. No product code was changed.

## What the UI prints

`MonitorInfo.name` is the only string in:

- the card title (`crates/ferrostage-ui/src/display_settings.rs:299`)
- the application-window chips (`:397`)
- the PDF chips (`:433`)
- the Identify subtitle (`crates/ferrostage-ui/src/identify.rs:26`)

The card body already appends resolution, primary, and role. The chips the user boxed do not. The modal in the screenshot is the compact display panel (`presenter.rs:527` passes `detailed = false`), so it also omits the topology diagram and the identity-source line. The connection-confidence and GDI-ratio line still renders.

`numbered()` sorts by `MonitorId` (`display_settings.rs:94-102`). On this machine that order is BKX, BOE, RTK, while the desktop left-to-right order is BOE, BKX, RTK.

Apply copies `monitor.name` into `friendly_name` (`display_settings.rs:79`). Role matching uses `id` only.

## Why the string is identical

`enumerate()` copies `DISPLAY_DEVICE.DeviceString` (`crates/ferrostage-platform/src/windows.rs:73`). For these three active monitors that value is the generic monitor driver's description.

Registry `DeviceDesc` on each active instance:

```
@monitor.inf,%pnpmonitor.devicedesc%;Generic PnP Monitor
```

So Windows itself stores “Generic PnP Monitor” as the driver description. Device Manager's friendly name is a different property.

Active instances only (Status OK), matched to the device paths already stored as `MonitorId`:

| Device path token | Geometry from the saved log | Registry FriendlyName (unresolved) | PnP / WMI resolved name |
| --- | --- | --- | --- |
| `DISPLAY#BKX2700#5&16631d0&0&UID8451` | 2560×1440, primary, rotation 0, position `(0, 0)`, `\\.\DISPLAY2` | `@System32\drivers\dxgkrnl.sys,#303;Generic Monitor (%1);(BKX270)` | Generic Monitor (BKX270). EDID user name `BKX270`, manufacturer BKX, product 2700, 2022 week 32 |
| `DISPLAY#BOE0D5A#5&16631d0&0&UID8448` | 2560×1600, rotation 0, position `(-2560, -169)`, `\\.\DISPLAY1` | `@System32\drivers\dxgkrnl.sys,#304;Integrated Monitor (%1);(NE160QDM-NM9)` | Integrated Monitor (NE160QDM-NM9). EDID user name `NE160QDM-NM9`, manufacturer BOE, product 0D5A, 2024 week 43 |
| `DISPLAY#RTK2560#4&2878f4bf&0&UID69` | 1600×2560, rotation 270, position `(2560, -1142)` | `@System32\drivers\dxgkrnl.sys,#300;Generic Monitor` | Generic Monitor. EDID user name empty, manufacturer RTK, product 2560, year 2020 week 0 |

BOE `NE160QDM-NM9` is the internal 16-inch 2560×1600 panel. BKX is an external whose EDID name is only `BKX270`. RTK has no EDID name; the portrait geometry and the right-hand position are the useful discriminators.

Inactive PnP nodes (`BOE` UID0 / UID8388688, `RTK` UID4165, `IPS2800` / C280SCC) were not in the current three-monitor enumeration. Do not show them.

## What a label can use without new identity

Already on `MonitorInfo`: `native_size`, `position`, `rotation_degrees`, `is_primary`, `connection_name`. A pure function of the current list can say:

- `1 · 2560×1440 · 主屏`
- `2 · 2560×1600 · 左侧`
- `3 · 1600×2560 · 竖屏 · 右侧`

Relative position is derived by comparing `position` inside the current set. It is a view hint. It must not be persisted as identity, and it must not be described as a Windows Settings number. `connection_name` (`\\.\DISPLAY1` and so on) is a GDI connection, also not the Windows Settings number; prefer not to put it on the chip.

This layer works when every model name is missing or still generic, including two identical panels, as long as size or position differs. If size and position also match, the existing FerroStage number remains the last distinction and Identify still flashes that number.

## Optional model line, and why it is not free

The useful model strings already exist in Windows. They are not in `DeviceString`.

Pinned `winsafe 0.0.29`:

- `RegOpenKeyEx` and `RegGetValue` exist, but only behind the `advapi` feature. The workspace currently enables `gdi` only (`Cargo.toml` winsafe features). `gdi` pulls `user` and `kernel`, not `advapi`.
- No `SHLoadIndirectString`, `QueryDisplayConfig`, or `DisplayConfigGetDeviceInfo`.
- Adding a crate, an unsafe block, or a `winsafe` upgrade is out of scope for the first cut. Task 03 already rejected `display-info` because of the fxhash advisory. CCD remains documented in `.trellis/tasks/10-14-presenter-v0.3/research/runtime-feasibility.md` as a possible EDID source, not as an authorized dependency.

A best-effort registry read can open `HKLM\SYSTEM\CurrentControlSet\Enum\DISPLAY\{hardware-id}\{instance}` parsed from the device interface path we already store. The value must be parsed, not shown raw:

- `@dll,#id;Generic Monitor (%1);(BKX270)` → `Generic Monitor (BKX270)` → model token `BKX270`
- `@dll,#id;Integrated Monitor (%1);(NE160QDM-NM9)` → integrated flag + `NE160QDM-NM9`
- `@dll,#id;Generic Monitor` → no model; geometry label only

The English fallback after the first semicolon matched `Get-PnpDevice` on this machine. Resource IDs `#300` / `#303` / `#304` are observations, not a public contract; prefer the stored fallback text. A parse failure or access failure omits the model and must not fail `enumerate_monitors`.

Do not persist the serial. The RTK serial is a placeholder, and a serial does not help the speaker pick a screen.

If this source is chosen, `friendly_name` should store the short model (or stay on the driver description when there is none), not the composed chip sentence. Resolution and position change with mode and arrangement; they belong on the view label.

## Options

1. Geometry only. No platform or feature change. Chips become distinct on this machine. The speaker still never sees BKX270 or NE160QDM-NM9.
2. Geometry plus the parsed FriendlyName. Same machine then shows BKX270, the internal panel model, and a geometry-only label for RTK. Requires enabling `advapi` on the pinned crate and a best-effort lookup. Enumeration stays successful when the lookup fails.
3. User-typed aliases. Most reliable for “投影仪 / 观众左屏”, and a new editor plus a settings field. Not required to fix the identical driver description.

Option 2 is the recommendation. Option 1 is the fallback if registry access is rejected. Option 3 stays out of this task.
