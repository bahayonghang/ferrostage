# V15 F13 — desktop DC leak: measured root cause and bounded repair

The V14 detection failure has a concrete per-query resource leak in the pinned safe wrapper path. This is separate from F14 (AccessKit close panic), which remains unresolved in the product. Root owns the V14 native evidence and V15 native retest; this report does not declare Gate B or native acceptance complete.

## Evidence and ownership

- `native/v14-dark-matrix.stdout.log`: after about 36 minutes of a healthy three-monitor session, `GetDC(desktop)` fails at 07:58:08Z; Retry fails again at 07:58:40Z. The UI safely disables unavailable topology.
- Fresh same-executable `v14-core-relaunch` immediately detects three monitors. This distinguishes a process-local resource accumulation hypothesis from persistent device removal, but by itself does not prove a leak.
- `crates/ferrostage-platform/src/windows.rs::geometry` acquired `HWND::NULL.GetDC()` on each enumeration. `watch.rs` polls every 200 ms (plus work time).
- Pinned `winsafe 0.0.29`, `src/user/guards.rs::ReleaseDCGuard::drop`, first tests `self.hwnd.as_opt()`. NULL is excluded by `src/kernel/traits.rs::Handle::as_opt`, so its otherwise-valid DC is never passed to `ReleaseDC`. The wrapper owns both handles but discards this valid NULL-owner case.
- A zero error code is not success: the wrapper recognizes a NULL GetDC result as failure and reads LastError; `devices()` clears LastError before enumeration. GetDC does not promise a useful error code for this resource failure.

Microsoft requires common DC release on the acquiring thread; NULL HWND is a valid screen DC owner. See [GetDC](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getdc), [ReleaseDC](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-releasedc). No exact GDI quota was measured or assumed.

## Real native red → green

Test: `windows::tests::getdc_v15_real_owner_preserves_topology_and_enumeration_releases_gdi` in `crates/ferrostage-platform/src/windows/tests.rs`. It creates no window and moves nothing. A held real-owner DC keeps GetGuiResources above zero because winsafe treats zero as an error.

| Receipt | Real-owner candidate, 16 enumerations | Production enumeration, 64 calls | Result |
| --- | --- | --- | --- |
| `product-v15-getdc-red.json` | three monitors; GDI 2 → 2; full MonitorInfo equality | GDI 2 → 66 | actual assertion FAIL, exit 101 |
| `product-v15-getdc-green.json` | three monitors; GDI 1 → 1; full MonitorInfo equality | GDI 1 → 1 | PASS, exit 0 |

Each result compares complete MonitorInfo values: stable ID, connection/name, native dimensions, position, scale/scale source, rotation and primary flag. The red test observes exactly one leaked GDI object per production enumeration. The same test becomes flat after the production acquisition changes to `HWND::GetDesktopWindow().GetDC()`.

The complete topology comparison matters: a non-NULL DC enumerates the monitors intersecting its visible region, per [EnumDisplayMonitors](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enumdisplaymonitors). We did not infer three-monitor coverage from its name. The actual comparison passed on this three-monitor desktop. Existing device/geometry key matching still rejects incomplete enumeration; it never silently accepts a subset. Root must still confirm the real GPUI DPI context in V15. Physical unplug/reconnect/ports and real single/zero-monitor acceptance remain NOT_RUN.

## Product delta

Only `geometry`'s acquisition and its explanatory comment changed; the now-unused Handle import was removed. The safe wrapper now receives a non-NULL owning HWND and its existing guard calls ReleaseDC on scope exit, including an error return. Callback DCs do not escape. Device identity, topology failure handling, polling cadence and monitor-role logic are unchanged. No new unsafe code, dependency, sixth maintained package, policy waiver or log suppression was introduced.

The Windows resource regression runs in the default suite; it requires a real desktop and deliberately fails if no monitor can be enumerated. This is not a physical hotplug acceptance substitute.

## F14 boundary

The same V14 dark run subsequently panicked in AccessKit's `ClientToScreen(...).unwrap()` after Close. GDI exhaustion may have affected timing, but no causal proof links the panic to the leak; fixing F13 cannot establish F14 PASS. `product-v15-a11y-proposal.md` and its research-only patch document the distinct direct COM-provider lifetime path and alternatives. Product AccessKit/GPUI sources remain unchanged in V15.
