# Native research observations — 2026-10-14

These are read-only research observations, not implementation or passed gates.

- The current Windows session enumerates three screens through System.Windows.Forms (DISPLAY1/2/5, DISPLAY2 primary, DISPLAY5 portrait). Those bounds are not established as physical pixels or stable identities. Port changes/hot unplug and native application placement remain NOT_RUN.
- The pinned [display-info 0.5.9 Windows implementation](https://raw.githubusercontent.com/nashaofu/display-info/v0.5.9/src/windows/mod.rs) uses GDI names and DEVMODE geometry; its numeric ID comes from HMONITOR. Its `all()` logs and drops individual monitor-construction failures. Thus this source alone cannot prove trustworthy complete topology, persistent identity, or typed enumeration failures. Task03 must resolve these before accepting a wrapper.
- The [public DisplayInfo API](https://docs.rs/display-info/0.5.9/display_info/struct.DisplayInfo.html) exposes geometry, friendly name, scale, rotation and primary state; it does not establish a durable EDID identity or change events.
- Direct web reads of docs.rs source Cargo.toml/PDFium package and GPUI pinned Cargo.toml failed/cache-missed in this session. This does not prove the dependencies unavailable; inspect installed sources or try the package/repository via the corresponding task research gate.

No crate/lint/pin changes made by this research. Do not weaken forbid(unsafe_code), silently accept incomplete enumeration, or treat this as a passed native compile/runtime check.

## Additional safe-interface lead (not selected or compiled)

The local Cargo cache contains `winsafe-0.0.19` (MIT, package manifest inspected) under `C:/Users/lyh/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`. Its `src/user/funcs.rs:283` exposes safe `EnumDisplayDevices(device_name, flags)` returning iterator items `SysResult<&DISPLAY_DEVICE>`; `src/user/structs.rs:434` exposes safe DeviceName/DeviceID/DeviceString getters. Safe EnumDisplaySettings and HDC::EnumDisplayMonitors / HMONITOR::GetMonitorInfo wrappers also exist. No GetDpiForMonitor wrapper was found in that cached version.

Task03 can investigate safe device-interface IDs using EDD_GET_DEVICE_INTERFACE_NAME and cross-check complete active device enumeration against a geometry/scale provider, rejecting incomplete results. This may avoid a project-local unsafe-policy change, but the exact semantics, enabled features, dependency licenses, compilation and native results remain unverified. Do not infer correctness merely from safe method signatures or select cached 0.0.19 without evaluating compatibility.

The [Microsoft EnumDisplayDevicesW contract](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enumdisplaydevicesw) confirms that EDD_GET_DEVICE_INTERFACE_NAME returns the per-monitor device-interface name in DeviceID; enumerate attached adapters, then their monitor devices. This provides a GDI-to-device-interface link, not a guarantee of identity after a port change.

Cached winsafe 0.0.19's iterator reuses an internal DISPLAY_DEVICE and extends a reference lifetime in its implementation (`src/user/iterators.rs:6–42`). Do not retain yielded references across iteration or collect them; task03 must review the exact chosen wrapper version and immediately copy owned strings if using that documented iteration pattern. Search results currently expose winsafe versions 0.0.27–0.0.29 inconsistently; reconcile exact package sources rather than treating a latest-docs result as a pin.

The pinned [display-info Windows helpers](https://raw.githubusercontent.com/nashaofu/display-info/v0.5.9/src/windows/utils.rs) use DEVMODE for physical geometry. Scale first attempts GetDpiForMonitor when the process is DPI-aware, otherwise falls back to a device-capabilities ratio. Task03 must record the process DPI context and scale source/limitations; task07 must verify its native mapping with the actual GPUI process. A valid float alone does not establish correct mixed-DPI placement.

## PDFium prerequisite lookup (2026-10-14, not a runtime pass)

The exact [pdfium-render 0.9.4 documentation](https://docs.rs/pdfium-render/0.9.4/pdfium_render/) is available and names pdfium_7881. The [immutable upstream chromium/7881 release](https://github.com/bblanchon/pdfium-binaries/releases/tag/chromium/7881) was independently queried through the GitHub release API. Asset pdfium-win-x64.tgz is 3,733,154 bytes; publisher digest is `sha256:73cc0de638ac2095e7445bf56a38200a5b7c7ca0e9f4ba144598f2457377ac08`; download source is https://github.com/bblanchon/pdfium-binaries/releases/download/chromium/7881/pdfium-win-x64.tgz . Published timestamp: 2026-06-08T17:07:25Z.

The sandbox HTTP lookup initially failed DNS; the approved read-only network retry succeeded. No archive was downloaded or executed by this lookup. Task05 must independently hash the downloaded archive, inspect bundled notices and runtime identity, bind the exact library, and run the native fixtures/performance protocol. Do not treat publisher metadata as a passed runtime or license review.

The read-only machine inventory is saved in native-environment.json (Windows build 26300, Core Ultra 9 275HX, NVIDIA/Intel plus virtual display drivers). WMI AdapterRAM may be truncated and is not a VRAM measurement; adapter mode values do not prove per-monitor physical bounds, DPI, or active topology. Task03 must establish its own API mapping, and later GPU measurements must identify the actual rendering device.

## PDFium artifact prefetch (2026-10-14)

A later authorized prerequisite fetch downloaded the exact archive to ignored workspace `cache/pdfium-7881-win-x64/pdfium-win-x64.tgz`. SHA256 independently matches the publisher digest above. Archive entries were inspected before extraction into the same directory. VERSION is 151.0.7881.0; `bin/pdfium.dll` SHA256 is `79d4676b656cfb1abcea88f9ade3b4b0826c5200382db5f4ec72a636c598c118`. No library was loaded or executed.

The package LICENSE is the build-packaging MIT notice and points to `licenses/`; it is not a replacement for PDFium's own or other third-party notices. Those files are present, including pdfium, abseil, simdutf, llvm-libc, freetype, zlib, libjpeg_turbo, fast_float, agg23, libopenjpeg, icu, libtiff, libpng and lcms. The bundled freetype notice identifies the FreeType Project License. Task05 still owns the complete notice/license audit, runtime loading, dependency selection, native test and performance results. This cache is ignored, is not an application install, and must not be committed.

## Pinned GPUI startup lead (2026-10-14, no build)

Direct HTTPS reads of the v1.23.3 tag succeeded after the web reader cache missed. The [gpui manifest](https://raw.githubusercontent.com/zed-industries/zed/v1.23.3/crates/gpui/Cargo.toml) declares package version 0.2.2 and Apache-2.0; this is the allowed tagged Zed source, not permission to substitute crates.io gpui 0.2.x. The exact tag's [hello_world example](https://raw.githubusercontent.com/zed-industries/zed/v1.23.3/crates/gpui/examples/hello_world.rs) starts with `gpui_platform::application()`. Its companion [platform manifest](https://raw.githubusercontent.com/zed-industries/zed/v1.23.3/crates/gpui_platform/Cargo.toml) declares Apache-2.0 and selects gpui_windows on Windows; [platform source](https://raw.githubusercontent.com/zed-industries/zed/v1.23.3/crates/gpui_platform/src/gpui_platform.rs) constructs Application with the current platform. Read-only cached copies are under ignored cache/gpui-v1.23.3.

Task07 should account for this platform split when evaluating the two authorized dependency routes. No GPUI dependency has been selected, linked, built or launched by this lookup. The tagged example's unwrap/expect and upstream ignored graphical tests are not project error handling or acceptance policy.

## Exact tagged GPUI display mapping caveats

The [v1.23.3 Windows display implementation](https://raw.githubusercontent.com/zed-industries/zed/v1.23.3/crates/gpui_windows/src/display.rs) and [public platform API](https://raw.githubusercontent.com/zed-industries/zed/v1.23.3/crates/gpui/src/platform.rs) were read directly and cached under cache/gpui-v1.23.3. On this route, Windows DisplayId derives from the current HMONITOR; the UUID is computed from the GDI device-name buffer. It is not the device-interface identity produced by task03, despite the generic trait documentation describing a stable UUID. Do not persist either as MonitorId or assume port stability.

PlatformDisplay publicly exposes id/uuid/logical bounds/visible bounds, but not the Windows implementation's private scale_factor/physical_bounds. Windows obtains effective DPI internally and converts each monitor's physical position/size by its scale. WindowOptions does expose display_id, window_bounds, focus, show, titlebar and minimum size.

Task07 must choose and verify a unique mapping against this exact public surface (or independently inspect the authorized kit route). If an explicitly transient native handle field is needed from task03, synchronize the shared contract and add the affected platform regression first; do not reinterpret GdiDeviceCapsRatio as effective DPI or introduce project unsafe to access private fields. GPUI enumeration can filter construction failures, so count/set completeness must also be checked. No GPUI build or native window behavior has been verified by reading source.

## Development fixture tooling observation

Get-Command resolved pdflatex.exe, xelatex.exe and latexmk.exe under C:/texlive/2025/bin/windows. This is availability only; no fixture was compiled by the lookup. Tasks05/06 may use an independent development-time generator for their own checked-in test artifacts, including actual Beamer overlays/notes where needed. Never add a compiler subprocess or toolchain integration to the FerroStage product; do not infer PageLabel semantics or model correctness merely from successful compilation. Record exact source/PDF/font hashes and independently inspect the produced metadata/pixels.

## Task06 primary specification leads (planning only)

- [Beamer user guide 3.78](https://tug.ctan.org/macros/latex/contrib/beamer/doc/beameruserguide.pdf), section19.3/page199 and section22, confirms right-default second-screen notes and configurable left/top/bottom. Section14 movie options describes start/duration; the actual emitted PDF dictionaries still need independent inspection. The installed TeX package version may differ from this guide.
- [Upstream Beamer issue807](https://github.com/josephwright/beamer/issues/807) reports page-label discrepancies with pause and second-screen notes. Treat as a fixture-design warning, not proof that the installed version has that bug or that a successful compile establishes the expected map.
- [Adobe PDF reference1.6 table9.31](https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/pdfreference1.6.pdf) defines Movie Start/Duration time units: unscaled integers depend on the media's time scale; an explicit two-element time/scale array permits seconds conversion. Do not assume an integer is seconds. Mode Repeat is distinct from Once/Open/Palindrome.
- [Adobe PDF1.7 reference](https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/pdfreference1.7old.pdf) covers Movie/Screen annotations and rendition actions; the web reader refused the full32MB file. [Adobe-hosted ISO32000-2 FDIS](https://developer.adobe.com/document-services/docs/assets/5b15559b96303194340b99820d3a70fa/PDF_ISO_32000-2.pdf) indexes RichMediaSettings/Activation tables334–335. Task06 must read the exact relevant tables before implementing its declared subset; search snippets alone are not sufficient for nested field-path contracts.

No task06 product code or fixtures were created by this lookup. Task05 remains the sole active implementation. Do not inspect or translate Pympress or other GPL viewer implementation source.

### Task06 official reference cache available

The sandbox download failed DNS; an approved read-only retry fetched the complete Adobe-hosted PDF1.7 reference above into ignored `cache/pdf-spec/pdfreference1.7.pdf`. SHA256 `4AA598E7E2CC88867565FAE793DB38C140A10CA2B5551DCCDB45E2124282CE29`. The existing TeX Live pdftotext produced `cache/pdf-spec/pdfreference1.7.txt` (exit0); table9.14 begins near line34027 and table9.31 near34771 in that extraction. This is specification material for task06 research, not viewer implementation source or a model-test result. Both retrieval/extraction commands have finished; no lingering process.

### RichMedia extension source resolved

The earlier Adobe-hosted ISO32000-2 FDIS search result returned HTTP404 when opened, so it is not a verified readable source. The [PDF Association specification archive](https://pdfa.org/resource/pdf-specification-archive/) links an archived Adobe ExtensionLevel3 supplement. Its exact Adobe PDF was retrieved through [the linked Wayback capture](https://web.archive.org/web/20220306152229id_/https://www.adobe.com/content/dam/acom/en/devnet/pdf/adobe_supplement_iso32000.pdf), saved as ignored `cache/pdf-spec/adobe-supplement-extension3.pdf`, SHA256 `638F531B57CEB50B4F0B86A6740A57438CCECB0E434E32F0209D9C8200ECC44B`. pdftotext exit0; title confirms Adobe Supplement, BaseVersion1.7, ExtensionLevel3, June2008/SDK9.0. Extraction `adobe-supplement-extension3.txt` includes RichMediaActivation table9.50a near line3663 and the configuration/instance dictionaries later. Task06 can read the exact declared subset from this primary document without relying on a dead FDIS link or other viewer source. Retrieval/extraction finished; no running commands.

## Task07 native visual verification tooling

The available computer-use skill was read at `C:/Users/lyh/.codex/plugins/cache/openai-bundled/computer-use/26.1007.21434/skills/computer-use/SKILL.md`, together with guidance/API/confirmation docs. Its supported `@oai/sky` package imports successfully in the main node_repl session. This is capability initialization only: no app/window inventory, screenshot, input or native acceptance has run. Main will coordinate the single Windows UI automation controller after task07 builds; use actual returned window objects and observe→oneaction→refresh. Do not create custom PowerShell UI Automation/capture helpers or mix them with this skill. This sky entrypoint is distinct from the browser-only cua_repl native-disabled surface. Physical unplug/port-change remains unavailable as confirmed by the user.

### Task07 mapping option to evaluate before expanding platform API

Current task03 MonitorInfo already exposes connection_name, the owned GDI connection name for that enumeration. The cached pinned GPUI source computes its public display UUID from the complete MONITORINFOEXW szDevice UTF-16 buffer using big-endian bytes and UUIDv5 DNS namespace. Task07 may evaluate reproducing that explicitly pinned transient mapping from the GDI name plus exact buffer width/initialization, with unique/full-set checks and actual native verification, before adding a new platform handle field. This is a research option only, not a selected API: the handle bridge is also possible through safe winsafe Handle traits. Neither computed UUID nor HMONITOR may become persistent MonitorId, and no geometry fallback may treat GdiDeviceCapsRatio as effective DPI. The current source is in cache/gpui-v1.23.3/windows-display.rs; source provenance is linked above.

## Task07 current RustSec classification

The current official advisory pages were opened on2026-10-14: [paste RUSTSEC-2024-0436](https://rustsec.org/advisories/RUSTSEC-2024-0436.html), [rustybuzz RUSTSEC-2026-0206](https://rustsec.org/advisories/RUSTSEC-2026-0206.html), and [ttf-parser RUSTSEC-2026-0192](https://rustsec.org/advisories/RUSTSEC-2026-0192.html). All three are INFO Unmaintained with no patched versions. They identify alternative crates (pastey/with_builtin_macros, harfrust, skrifa), not a verified drop-in update within the locked GPUI route. Do not relabel these as vulnerability findings or assume alternatives can replace transitive dependencies without source/API changes. Task07 isolated exact-tag policy audit fails under the unchanged project policy; evaluate the other already-authorized route before declaring the dependency gate blocked.
