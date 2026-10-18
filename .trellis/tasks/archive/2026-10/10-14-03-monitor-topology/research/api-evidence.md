# Safe API and dependency decision — task 03

Final selection: exact Windows-only `winsafe =0.0.29`, MIT, declared MSRV1.87, features gdi (includes user/kernel). Cargo.lock provides registry checksum; dependency-source-sha256.json fingerprints the actually inspected cache sources. No new project unsafe code or lint-policy changes.

## Interfaces inspected

- [winsafe 0.0.29 package](https://docs.rs/crate/winsafe/0.0.29): cached Cargo.toml confirms version/license/MSRV/features. Safe functions were inspected in user/funcs.rs, user/handles/hdc.rs, user/handles/hwnd.rs, user/handles/hmonitor.rs, gdi/handles/hdc.rs and user/structs.rs.
- [EnumDisplayDevicesW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enumdisplaydevicesw): attached adapters followed by active monitor children; EDD_GET_DEVICE_INTERFACE_NAME returns per-monitor interface name. It is a connection-scoped key, not a portable EDID serial or port-independence guarantee.
- winsafe's iterator returns borrowed DISPLAY_DEVICE data backed by one reused buffer (user/iterators.rs). Implementation copies fields immediately, never retains yielded references or collects them. Nested calls do not alias the same iterator buffer. Windows LastError is cleared before a query and after nested successful queries because the wrapper consults it at end-of-list.
- [EnumDisplaySettingsW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enumdisplaysettingsw) states that output uses physical pixels, independent of DPI virtualization. The adapter requires valid POSITION/DISPLAYORIENTATION/PELSWIDTH/PELSHEIGHT fields and maps supported rotation values.
- [EnumDisplayMonitors](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enumdisplaymonitors) documents GetDC(NULL) + non-null DC enumeration of the virtual screen and per-monitor callback DCs. winsafe owns the desktop ReleaseDC guard; callback handles do not escape. No hidden HWND is created.
- [GetMonitorInfoW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmonitorinfow) supplies the GDI name and primary flag. Every callback result is retained as Result and collected; no single-monitor failure is swallowed. Device enumeration before/after must agree and the entire GDI set must correspond one-to-one to geometry.
- [GetDeviceCaps](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-getdevicecaps): use the callback's per-monitor DC for DESKTOPHORZRES/HORZRES. Positive physical width must match DEVMODE; ratio is labeled GdiDeviceCapsRatio. It does not prove effective DPI or the UI's coordinate convention. Process awareness is not modified or directly measured. 07 must verify the actual GPUI process, and must not interpret 1.0 as a 100% desktop setting.

No usable safe standalone display-change source was established without adding a window, so the already approved explicit 200 ms polling fallback is used. Pure fake-time tests prove coalescing/error recovery; a native static probe proves actual enumeration/initial delivery/cancellation on this session only. They do not prove physical hotplug latency.

## Independent wrapper audit

The pinned local source `src/macros/consts.rs:85` defines `has(other)` as `(self.0 & other.0) != 0`. It tests any bit, not all bits. Independent review caught the adapter's combined DEVMODE-field mask misuse and changed the required-fields check to `(fields & required) == required`, with each of the four fields omitted in a regression. Other adapter `has` calls test one bit. The full mode remains accepted with extra flags. Fresh native/CI results are in independent-check.md; independent-dependency-source-sha256.json records this ninth inspected dependency file in addition to the original eight.

## Rejected candidate

`display-info =0.5.9` (Apache-2.0) was initially combined with complete device enumeration to detect its swallowed per-monitor errors. Workspace tests passed, but cargo deny rejected its Windows fxhash0.2.1 dependency under [RUSTSEC-2025-0057](https://rustsec.org/advisories/RUSTSEC-2025-0057). The candidate was removed from manifest/lock; no advisory ignore, license exception, local unsafe override, shim or vendor fork was added. Final cargo deny passes. Its earlier probe/log is labeled superseded.
